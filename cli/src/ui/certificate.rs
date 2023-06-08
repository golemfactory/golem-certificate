use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::SignedCertificate;
use tui::{layout::{Direction, Layout, Rect, Constraint}, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::certificate_to_string,
    editors::{
        EditorComponent, EditorEventResult,
        permission::PermissionEditor,
        validity_period::ValidityPeriodEditor,
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

enum ActiveEditor {
    Permissions,
    ValidityPeriod,
}

pub struct CertificateEditor {
    active_editor: ActiveEditor,
    permissions_editor: PermissionEditor,
    validity_period_editor: ValidityPeriodEditor,
}

impl CertificateEditor {
    pub fn new() -> Self {
        let mut permissions_editor = PermissionEditor::new(None);
        permissions_editor.enter_from_top();
        Self {
            active_editor: ActiveEditor::Permissions,
            permissions_editor: permissions_editor,
            validity_period_editor: ValidityPeriodEditor::new(None),
        }
    }
}

impl Component for CertificateEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match self.active_editor {
            ActiveEditor::Permissions =>
                match self.permissions_editor.handle_key_event(key_event) {
                    EditorEventResult::ExitTop => self.permissions_editor.enter_from_top(),
                    EditorEventResult::ExitBottom => {
                        self.validity_period_editor.enter_from_top();
                        self.active_editor = ActiveEditor::ValidityPeriod;
                    }
                    _ => {},
                },
            ActiveEditor::ValidityPeriod =>
                match self.validity_period_editor.handle_key_event(key_event) {
                    EditorEventResult::ExitTop => {
                        self.permissions_editor.enter_from_below();
                        self.active_editor = ActiveEditor::Permissions;
                    }
                    EditorEventResult::ExitBottom => self.validity_period_editor.enter_from_below(),
                    _ => {},
                }
        }
        Ok(ComponentStatus::Active)
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(self.permissions_editor.calculate_render_height() as u16),
                Constraint::Max(self.validity_period_editor.calculate_render_height() as u16),
                Constraint::Min(0),
            ])
            .split(area);
        let permissions_cursor = self.permissions_editor.render(chunks[0], buf);
        let validity_period_cursor = self.validity_period_editor.render(chunks[1], buf);
        permissions_cursor.or(validity_period_cursor)
    }
}
