use std::fmt::Write;

use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use tui::{layout::{Constraint, Direction, Layout, Rect}, buffer::Buffer, widgets::{Clear, Paragraph, Widget}, text::Span};

use super::{
    component::*,
    modal::ModalMessage,
    text_input::TextInput,
    util::{default_style, highlight_style},
};

mod key_usage;
pub use key_usage::KeyUsageEditor;

mod node_id;
pub use node_id::NodeIdEditor;

mod permissions;
pub use permissions::PermissionsEditor;

mod validity_period;
pub use validity_period::ValidityPeriodEditor;

pub enum EditorEventResult {
    ExitTop,
    ExitBottom,
    KeepActive,
    Escaped,
    Inactive,
}

pub trait EditorGroup {
    fn get_editor_group_state(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>);

    fn init(&mut self) {
        let (active_editor_idx, mut editors) = self.get_editor_group_state();
        editors[*active_editor_idx].enter_from_top();
    }
}

impl Component for dyn EditorGroup {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let (active_editor_idx, mut editors) =
            self.get_editor_group_state();
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
            _ => {},
        }
        Ok(ComponentStatus::Active)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let (_, mut editors) = self.get_editor_group_state();
        let mut constraints = editors.iter()
            .map(|editor| Constraint::Max(editor.calculate_render_height() as u16 + 1))
            .collect::<Vec<_>>();
        constraints.push(Constraint::Min(0));
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        editors.iter_mut()
            .enumerate()
            .map(|(idx, editor)| editor.render(chunks[idx], buf))
            .fold(None, |acc, cursor| acc.or(cursor))
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
    fn get_editor(&mut self) -> Option<&mut TextInput>;
    fn get_parse_error(&mut self) -> Option<&mut ModalMessage>;

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let text = {
            let mut text = String::new();
            self.get_text_output(&mut text);
            text
        };
        if self.get_editor().is_some() || self.get_highlight().is_none() {
            Paragraph::new(text)
                .alignment(tui::layout::Alignment::Left)
                .style(default_style())
                .render(area, buf);

            let highlight = self.get_highlight();
            let highlight_prefix = self.get_highlight_prefix();
            if let Some(url_editor) = self.get_editor() {
                let prefix = highlight_prefix.expect("Cannot have text input active without highlight in the component") as u16;
                let editor_area = Rect {
                    x: area.x + prefix,
                    y: area.y + highlight.expect("Cannot have text input active without highlight in the component") as u16,
                    width: area.width.saturating_sub(prefix),
                    height: 1.min(area.height),
                };
                Clear.render(editor_area, buf);
                let editor_cursor = url_editor.render(editor_area, buf);
                if let Some(parse_error) = self.get_parse_error() {
                    parse_error.render(area, buf)
                } else {
                    editor_cursor
                }
            } else {
                None
            }
        } else {
            render_with_highlight(&text, self.get_highlight().unwrap(), self.get_highlight_prefix().unwrap(), area, buf);
            None
        }
    }
}

fn render_with_highlight(text: &String, highlight: usize, prefix: usize, area: Rect, buf: &mut Buffer) {
    let pre = text.lines().take(highlight).collect::<Vec<_>>().join("\n");
    let highlighted = text.lines().skip(highlight).take(1).collect::<String>();
    let post = text.lines().skip(highlight + 1).collect::<Vec<_>>().join("\n");
    let highlight_area = adjust_render_area(&area, &pre);
    Paragraph::new(pre)
        .alignment(tui::layout::Alignment::Left)
        .style(default_style())
        .render(area, buf);
    if let Some(mut area) = highlight_area {
        let post_area = adjust_render_area(&area, &highlighted);
        area.height = area.height.min(1);
        let (highlight_prefix, highlighted_text) = {
            if highlighted.is_empty() {
                ("      ".into(), "<Add another URL>".into())
            } else {
                let highlight_prefix = highlighted.chars().take(prefix).collect::<String>();
                let highlighted_text = highlighted.chars().skip(prefix).collect::<String>();
                (highlight_prefix, highlighted_text)
            }
        };
        let text_area = Rect {
            x: area.x + prefix as u16,
            y: area.y,
            width: area.width.saturating_sub(prefix as u16),
            height: area.height,
        };
        Paragraph::new(highlight_prefix)
            .alignment(tui::layout::Alignment::Left)
            .style(default_style())
            .render(area, buf);
        let span = Span::styled(highlighted_text, highlight_style());
        Paragraph::new(span)
            .alignment(tui::layout::Alignment::Left)
            .render(text_area, buf);
        if let Some(area) = post_area {
            Paragraph::new(post)
                .alignment(tui::layout::Alignment::Left)
                .style(default_style())
                .render(area, buf);
        }
    } else {
        unreachable!("Highlighted part is not within render area");
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
