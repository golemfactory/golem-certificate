use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::SignedCertificate;
use tui::{layout::Rect, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::certificate_to_string,
    editors::{
        EditorComponent,
        permission::PermissionEditor,
    },
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth,
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

pub struct CertificateEditor {
    permissions_editor: PermissionEditor,
}

impl CertificateEditor {
    pub fn new() -> Self {
        let mut permissions_editor = PermissionEditor::new(None);
        permissions_editor.enter_from_top();
        Self {
            permissions_editor: permissions_editor,
        }
    }
}

impl Component for CertificateEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match self.permissions_editor.handle_key_event(key_event) {
            super::editors::EditorEventResult::ExitTop => self.permissions_editor.enter_from_top(),
            super::editors::EditorEventResult::ExitBottom => self.permissions_editor.enter_from_below(),
            _ => {},
        }
        Ok(ComponentStatus::Active)
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        self.permissions_editor.render(area, buf)
    }
}
