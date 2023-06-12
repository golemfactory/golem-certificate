use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::{SignedCertificate, schemas::{certificate::Certificate, signature}};
use tui::{layout::Rect, widgets::{StatefulWidget, Block, BorderType, Borders, Widget}};

use super::{
    component::*,
    display_details::certificate_to_string,
    editors::*,
    modal::{ModalMultipleChoice, ModalWindow},
    multiple_choice::{EXIT_WITHOUT_SAVE, MultipleChoice, SIGN_OR_TEMPLATE},
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth, identity_area,
    },
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

    pub fn new_with_defaults(cert: &SignedCertificate) -> Self {
        let text = certificate_to_string(cert, 2, false);
        let (calculate_height, calculate_width) = identity_area();
        Self {
            render_state: ScrollableTextState::new(text),
            calculate_height,
            calculate_width,
        }
    }

    pub fn get_text_height(&self) -> usize {
        self.render_state.get_text_height()
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
struct CertificateDocumentEditor {
    key_usage_editor: KeyUsageEditor,
    permissions_editor: PermissionsEditor,
    public_key_editor: KeyEditor,
    subject_editor: SubjectEditor,
    validity_period_editor: ValidityPeriodEditor,
}

impl CertificateDocumentEditor {
    fn get_data(&self) -> Result<serde_json::Value> {
        if let Some(key) = self.public_key_editor.get_key() {
            let cert = Certificate {
                key_usage: self.key_usage_editor.get_key_usage(),
                permissions: self.permissions_editor.get_permissions(),
                public_key: key,
                subject: self.subject_editor.get_subject(),
                validity_period: self.validity_period_editor.get_validity_period(),
            };
            Ok(serde_json::to_value(cert)?)
        } else {
            anyhow::bail!("No public key")
        }
    }

    fn editors_mut(&mut self) -> Vec<&mut dyn EditorComponent> {
        vec![
            &mut self.subject_editor,
            &mut self.permissions_editor,
            &mut self.validity_period_editor,
            &mut self.public_key_editor,
            &mut self.key_usage_editor,
        ]
    }
}

pub struct CertificateEditor {
    active_editor_idx: usize,
    document: CertificateDocumentEditor,
    save_or_template: MultipleChoice,
    signature_editor: Option<(ModalWindow, SignatureEditor)>,
    popup: Option<ModalMultipleChoice>,
}

impl CertificateEditor {
    pub fn new() -> Self {
        let mut save_or_template = MultipleChoice::new(SIGN_OR_TEMPLATE, 0);
        save_or_template.active = false;

        let mut editor = Self {
            active_editor_idx: 0,
            document: CertificateDocumentEditor::default(),
            save_or_template,
            signature_editor: None,
            popup: None,
        };
        editor.init();
        editor
    }

    fn add_signature_editor(&mut self) {
        let mut signature_editor = SignatureEditor::new(true);
        signature_editor.init();
        let modal_window = ModalWindow::new("Signature details");
        self.signature_editor = Some((modal_window, signature_editor));
    }
}

impl EditorGroup for CertificateEditor {
    fn editor_group_state_mut(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>) {
        let mut editors = self.document.editors_mut();
        editors.push(&mut self.save_or_template);
        (&mut self.active_editor_idx, editors)
    }
}

impl Component for CertificateEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(popup) = self.popup.as_mut() {
            match popup.handle_key_event(key_event)? {
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
            }
        } else if let Some((_, signature_editor)) = self.signature_editor.as_mut() {
            let editor: &mut dyn EditorGroup = signature_editor;
            match editor.handle_key_event(key_event)? {
                ComponentStatus::Active => Ok(ComponentStatus::Active),
                ComponentStatus::Escaped => {
                    self.signature_editor = None;
                    Ok(ComponentStatus::Active)
                }
                ComponentStatus::Closed => {
                    // TODO handle signing
                    Ok(ComponentStatus::Active)
                }
            }
        } else {
            let editor_group: &mut dyn EditorGroup = self;
            match editor_group.handle_key_event(key_event)? {
                ComponentStatus::Active => {},
                ComponentStatus::Escaped => {
                    self.popup = Some(ModalMultipleChoice::new(
                        "Exit without saving?",
                        "Changes will be lost.",
                        EXIT_WITHOUT_SAVE,
                        1,
                    ));
                }
                ComponentStatus::Closed => {
                    if self.save_or_template.get_selected() == SIGN_OR_TEMPLATE[0] {
                        self.add_signature_editor();
                    } else {
                        // TODO handle signing or saving as template
                    }
                }
            }
            Ok(ComponentStatus::Active)
        }
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        let block = Block::default()
            .title("Certificate editor")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(default_style());

        let editor_area = block.inner(area);
        block.render(area, buf);

        let editor_group: &mut dyn EditorGroup = self;
        let mut cursor = editor_group.render(editor_area, buf);
        if let Some((modal_window, signature_editor)) = self.signature_editor.as_mut() {
            let inner_area = modal_window.render(editor_area, buf, area.height.saturating_sub(4), area.width.saturating_sub(4));
            let editor: &mut dyn EditorGroup = signature_editor;
            cursor = editor.render(inner_area, buf);
        }
        if let Some(popup) = self.popup.as_mut() {
            cursor = popup.render(editor_area, buf);
        }
        cursor
    }
}
