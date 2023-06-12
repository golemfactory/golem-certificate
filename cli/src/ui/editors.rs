use std::fmt::Write;

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use tui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, buffer::Buffer, widgets::{Clear, Paragraph, Widget}, text::Span};

use super::{
    component::*,
    modal::ModalMessage,
    text_input::TextInput,
    util::{default_style, highlight_style},
};

mod key;
pub use key::KeyEditor;

mod key_usage;
pub use key_usage::KeyUsageEditor;

mod node_id;
pub use node_id::NodeIdEditor;

mod permissions;
pub use permissions::PermissionsEditor;

mod signature;
pub use signature::SignatureEditor;

mod subject;
pub use subject::SubjectEditor;

mod validity_period;
pub use validity_period::ValidityPeriodEditor;

pub enum EditorEventResult {
    Closed,
    ExitTop,
    ExitBottom,
    KeepActive,
    Escaped,
    Inactive,
}

pub trait EditorGroup {
    fn editor_group_state_mut(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>);

    fn init(&mut self) {
        let (active_editor_idx, mut editors) = self.editor_group_state_mut();
        editors[*active_editor_idx].enter_from_top();
    }
}

impl Component for dyn EditorGroup {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let (active_editor_idx, mut editors) =
            self.editor_group_state_mut();
        match editors[*active_editor_idx].handle_key_event(key_event) {
            EditorEventResult::ExitTop => {
                if *active_editor_idx > 0 {
                    *active_editor_idx -= 1;
                    editors[*active_editor_idx].enter_from_below();
                } else {
                    editors[*active_editor_idx].enter_from_top();
                }
            }
            EditorEventResult::ExitBottom => {
                if *active_editor_idx < editors.len() - 1 {
                    *active_editor_idx += 1;
                    editors[*active_editor_idx].enter_from_top();
                } else {
                    editors[*active_editor_idx].enter_from_below();
                }
            }
            EditorEventResult::Escaped => return Ok(ComponentStatus::Escaped),
            EditorEventResult::Closed => return Ok(ComponentStatus::Closed),
            _ => {},
        }
        Ok(ComponentStatus::Active)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let (active, mut editors) = self.editor_group_state_mut();
        let mut constraints = editors.iter()
            .map(|editor| Constraint::Max(editor.calculate_render_height() as u16 + 1))
            .collect::<Vec<_>>();
        constraints.push(Constraint::Min(0));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        let editor_cursor = editors.iter_mut()
            .enumerate()
            .map(|(idx, editor)| editor.render(chunks[idx], buf))
            .fold(None, |acc, cursor| acc.or(cursor));
        let modal_cursor = editors[*active].render_modal(area, buf);
        editor_cursor.or(modal_cursor)
    }
}

pub trait EditorComponent {
    fn enter_from_below(&mut self);
    fn enter_from_top(&mut self);
    fn get_highlight(&self) -> Option<usize>;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult;
    fn calculate_render_height(&self) -> usize;
    fn get_text_output(&self, text: &mut String);
    fn get_highlight_prefix(&self) -> Option<usize>;

    fn get_editor(&mut self) -> Option<&mut TextInput> {
        None
    }

    fn get_error_message(&mut self) -> Option<&mut ModalMessage> {
        None
    }

    fn get_empty_highlight_filler(&self) -> (String, String) {
        (String::new(), String::new())
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let text = {
            let mut text = String::new();
            self.get_text_output(&mut text);
            text
        };
        if self.get_editor().is_some() || self.get_highlight().is_none() {
            Paragraph::new(text)
                .alignment(Alignment::Left)
                .style(default_style())
                .render(area, buf);

            let highlight = self.get_highlight();
            let highlight_prefix = self.get_highlight_prefix();
            if let Some(editor) = self.get_editor() {
                let prefix = highlight_prefix.expect("Cannot have text input active without highlight in the component") as u16;
                let editor_area = Rect {
                    x: area.x + prefix,
                    y: area.y + highlight.expect("Cannot have text input active without highlight in the component") as u16,
                    width: area.width.saturating_sub(prefix),
                    height: 1.min(area.height),
                };
                Clear.render(editor_area, buf);
                Component::render(editor, editor_area, buf)
            } else {
                None
            }
        } else {
            self.render_with_highlight(&text, area, buf);
            None
        }
    }

    fn render_modal(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        if let Some(parse_error) = self.get_error_message() {
            parse_error.render(area, buf)
        } else {
            None
        }
    }

    fn render_with_highlight(&self, text: &String, area: Rect, buf: &mut Buffer) {
        let highlight = self.get_highlight().expect("Cannot render with highlight without highlight in the component");
        let prefix = self.get_highlight_prefix().expect("Cannot render with highlight without highlight in the component");
        let skip = if area.height < highlight as u16 + 1 {
            1 + highlight - area.height as usize
        } else {
            0
        };
        let pre = text.lines().skip(skip).take(highlight).collect::<Vec<_>>().join("\n");
        let highlighted = text.lines().skip(highlight + skip).take(1).collect::<String>();
        let post = text.lines().skip(highlight + skip + 1).collect::<Vec<_>>().join("\n");
        let highlight_area = adjust_render_area(&area, &pre);
        Paragraph::new(pre)
            .alignment(Alignment::Left)
            .style(default_style())
            .render(area, buf);
        if let Some(mut area) = highlight_area {
            let (highlight_prefix, highlighted_text) = {
                if highlighted.is_empty() {
                    self.get_empty_highlight_filler()
                } else {
                    let highlight_prefix = highlighted.chars().take(prefix).collect::<String>();
                    let highlighted_text = highlighted.chars().skip(prefix).collect::<String>();
                    (highlight_prefix, highlighted_text)
                }
            };
            let post_area = adjust_render_area(&area, &highlighted_text);
            area.height = area.height.min(1);
            let text_area = Rect {
                x: area.x + prefix as u16,
                y: area.y,
                width: area.width.saturating_sub(prefix as u16),
                height: area.height,
            };
            Paragraph::new(highlight_prefix)
                .alignment(Alignment::Left)
                .style(default_style())
                .render(area, buf);
            let span = Span::styled(highlighted_text, highlight_style());
            Paragraph::new(span)
                .alignment(Alignment::Left)
                .render(text_area, buf);
            if let Some(area) = post_area {
                Paragraph::new(post)
                    .alignment(Alignment::Left)
                    .style(default_style())
                    .render(area, buf);
            }
        } else {
            unreachable!("Highlighted part is not within render area");
        }
    }
}

fn adjust_render_area(area: &Rect, text: &String) -> Option<Rect> {
    let text_height = text.lines().count() as u16;
    let adjusted_area = Rect {
        x: area.x,
        y: area.y + text_height,
        width: area.width,
        height: area.height.saturating_sub(text_height),
    };
    if adjusted_area.height > 0 {
        Some(adjusted_area)
    } else {
        None
    }
}
