use std::fmt::Write;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Paragraph, Widget},
};

use super::{
    component::*,
    editors::{EditorComponent, EditorEventResult},
    modal::ModalMessage,
    util::default_style,
};

pub struct TextInput {
    pub active: bool,
    cursor_position: usize,
    masked: bool,
    max_length: usize,
    text: String,
}

impl TextInput {
    pub fn new(max_length: usize, masked: bool) -> Self {
        Self {
            active: true,
            cursor_position: 0,
            masked,
            max_length,
            text: String::new(),
        }
    }

    pub fn get_text_for_display(&self) -> String {
        if self.masked {
            self.text.chars().map(|_| '*').collect()
        } else {
            self.text.clone()
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.cursor_position = text.len();
        self.text = text;
    }

    pub fn get_text(&self) -> &String {
        &self.text
    }

    fn text_editing_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                if self.text.len() < self.max_length {
                    self.text.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_position < self.text.len() {
                    self.cursor_position += 1;
                }
            }
            KeyCode::Left => {
                self.cursor_position = self.cursor_position.saturating_sub(1);
            }
            KeyCode::Backspace => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.text.remove(self.cursor_position);
                }
            }
            KeyCode::Delete => {
                if self.cursor_position < self.text.len() {
                    self.text.remove(self.cursor_position);
                }
            }
            _ => (),
        }
    }
}

impl Component for TextInput {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            KeyCode::Enter => ComponentStatus::Closed,
            _ => {
                self.text_editing_key_event(key_event);
                ComponentStatus::Active
            }
        };
        Ok(status)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        Paragraph::new(self.get_text_for_display())
            .alignment(Alignment::Left)
            .style(default_style())
            .render(area, buf);
        if self.active && self.cursor_position < area.width as usize {
            Some(CursorPosition {
                x: area.x + self.cursor_position as u16,
                y: area.y,
            })
        } else {
            None
        }
    }
}

impl EditorComponent for TextInput {
    fn enter_from_below(&mut self) {
        self.active = true;
    }

    fn enter_from_top(&mut self) {
        self.active = true;
    }

    fn get_highlight(&self) -> Option<usize> {
        None
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        match key_event.code {
            KeyCode::Esc => EditorEventResult::Escaped,
            KeyCode::Enter => EditorEventResult::Closed,
            KeyCode::Down => {
                self.active = false;
                EditorEventResult::ExitBottom
            }
            KeyCode::Up => {
                self.active = false;
                EditorEventResult::ExitTop
            }
            _ => {
                self.text_editing_key_event(key_event);
                EditorEventResult::KeepActive
            }
        }
    }

    fn calculate_render_height(&self) -> usize {
        1
    }

    fn get_text_output(&self, text: &mut String) {
        writeln!(text, "{}", self.get_text_for_display()).unwrap();
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        None
    }

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
        Component::render(self, area, buf)
    }
}
