use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::SignedNodeDescriptor;
use tui::{buffer::Buffer, layout::{Rect, Layout, Direction, Constraint}, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::node_descriptor_to_string,
    editors::*,
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth,
    },
};

pub struct SignedNodeDescriptorDetails {
    calculate_height: CalculateHeight,
    calculate_width: CalculateWidth,
    render_state: ScrollableTextState,
}

impl SignedNodeDescriptorDetails {
    pub fn new(
        node_descriptor: &SignedNodeDescriptor,
        indent: usize,
        detailed_signer: bool,
        (calculate_height, calculate_width): AreaCalculators,
    ) -> Self {
        let text = node_descriptor_to_string(node_descriptor, indent, detailed_signer);
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

impl Component for SignedNodeDescriptorDetails {
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

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
        None
    }
}

impl SizedComponent for SignedNodeDescriptorDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        (
            (self.calculate_height)(area.height),
            (self.calculate_width)(area.width),
        )
    }
}

pub struct NodeDescriptorEditor {
    active: Editor,
    node_id: NodeIdEditor,
    permissions: PermissionEditor,
    validity_period: ValidityPeriodEditor,
}

impl NodeDescriptorEditor {
    pub fn new() -> Self {
        let mut node_id = NodeIdEditor::new();
        node_id.enter_from_top();
        Self {
            active: Editor::NodeId,
            node_id: node_id,
            permissions: PermissionEditor::new(None),
            validity_period: ValidityPeriodEditor::new(None),
        }
    }
}

impl Component for NodeDescriptorEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match self.active {
            Editor::NodeId => {
                match self.node_id.handle_key_event(key_event) {
                    EditorEventResult::Escaped => Ok(ComponentStatus::Escaped),
                    event => {
                        match event {
                            EditorEventResult::ExitTop => self.node_id.enter_from_top(),
                            EditorEventResult::ExitBottom => {
                                self.permissions.enter_from_top();
                                self.active = Editor::Permissions;
                            },
                            _ => {},
                        };
                        Ok(ComponentStatus::Active)
                    }
                }
            }
            Editor::Permissions => {
                match self.permissions.handle_key_event(key_event) {
                    EditorEventResult::Escaped => Ok(ComponentStatus::Escaped),
                    event => {
                        match event {
                            EditorEventResult::ExitTop => {
                                self.node_id.enter_from_below();
                                self.active = Editor::NodeId;
                            }
                            EditorEventResult::ExitBottom => {
                                self.validity_period.enter_from_top();
                                self.active = Editor::ValidityPeriod;
                            },
                            _ => {},
                        };
                        Ok(ComponentStatus::Active)
                    }
                }
            }
            Editor::ValidityPeriod => {
                match self.validity_period.handle_key_event(key_event) {
                    EditorEventResult::Escaped => Ok(ComponentStatus::Escaped),
                    event => {
                        match event {
                            EditorEventResult::ExitTop => {
                                self.permissions.enter_from_below();
                                self.active = Editor::Permissions;
                            }
                            EditorEventResult::ExitBottom => self.validity_period.enter_from_below(),
                            _ => {},
                        };
                        Ok(ComponentStatus::Active)
                    }
                }
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(self.node_id.calculate_render_height() as u16 + 1),
                Constraint::Max(self.permissions.calculate_render_height() as u16),
                Constraint::Max(self.validity_period.calculate_render_height() as u16),
                Constraint::Min(0),
            ])
            .split(area);
        let node_id_cursor = self.node_id.render(chunks[0], buf);
        let permissions_cursor = self.permissions.render(chunks[1], buf);
        let validity_period_cursor = self.validity_period.render(chunks[2], buf);
        node_id_cursor.or(permissions_cursor).or(validity_period_cursor)
    }
}
