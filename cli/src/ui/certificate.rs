use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::SignedCertificate;
use tui::{layout::Rect, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::certificate_to_string,
    editors::*,
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth,
    }, modal::{ModalMessage, ModalMultipleChoice}, multiple_choice::EXIT_WITHOUT_SAVE,
};

pub struct SignedCertificateDetails {
    calculate_height: CalculateHeight,
    calculate_width: CalculateWidth,
    render_state: ScrollableTextState,
}

impl SignedCertificateDetails {
    pub fn new(
        cert: &SignedCertificate,
        indent: usize,
        detailed_signer: bool,
        (calculate_height, calculate_width): AreaCalculators,
    ) -> Self {
        let text = certificate_to_string(cert, indent, detailed_signer);
        Self {
            render_state: ScrollableTextState::new(text),
            calculate_height,
            calculate_width,
        }
    }

    fn scrolling_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_sub(1);
            }
            KeyCode::Down => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_add(1);
            }
            _ => (),
        }
    }
}

impl Component for SignedCertificateDetails {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            _ => {
                self.scrolling_key_event(key_event);
                ComponentStatus::Active
            }
        };
        Ok(status)
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
        None
    }
}

impl SizedComponent for SignedCertificateDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        (
            (self.calculate_height)(area.height),
            (self.calculate_width)(area.width),
        )
    }
}

#[derive(Default)]
pub struct CertificateEditor {
    active_editor_idx: usize,
    key_usage_editor: KeyUsageEditor,
    permissions_editor: PermissionsEditor,
    public_key_editor: PublicKeyEditor,
    subject_editor: SubjectEditor,
    validity_period_editor: ValidityPeriodEditor,
    popup: Option<ModalMultipleChoice>,
}

impl CertificateEditor {
    pub fn new() -> Self {
        let mut certificate_editor = Self::default();
        certificate_editor.init();
        certificate_editor
    }
}

impl EditorGroup for CertificateEditor {
    fn get_editor_group_state(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>) {
        let editors: Vec<&mut dyn EditorComponent> = vec![
            &mut self.subject_editor,
            &mut self.permissions_editor,
            &mut self.validity_period_editor,
            &mut self.public_key_editor,
            &mut self.key_usage_editor,
        ];
        (&mut self.active_editor_idx, editors)
    }
}

impl Component for CertificateEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(popup) = self.popup.as_mut() {
            match popup.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => Ok(ComponentStatus::Active),
                    ComponentStatus::Escaped => {
                        self.popup = None;
                        Ok(ComponentStatus::Active)
                    }
                    ComponentStatus::Closed => {
                        let selected = popup.get_selected();
                        self.popup = None;
                        if selected == EXIT_WITHOUT_SAVE[0] {
                            Ok(ComponentStatus::Escaped)
                        } else {
                            Ok(ComponentStatus::Active)
                        }
                    }
                },
                Err(err) => return Err(err),
            }
        } else {
            let editor_group: &mut dyn EditorGroup = self;
            match editor_group.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Escaped => {
                        self.popup = Some(ModalMultipleChoice::new(
                            "Exit without saving?",
                            "Changes will be lost.",
                            EXIT_WITHOUT_SAVE,
                            1,
                        ));
                        Ok(ComponentStatus::Active)
                    },
                    status => Ok(status),
                },
                Err(err) => Err(err),
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        let editor_group: &mut dyn EditorGroup = self;
        let mut cursor = editor_group.render(area, buf);
        if let Some(popup) = self.popup.as_mut() {
            cursor = popup.render(area, buf);
        }
        cursor
    }
}
