use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Paragraph, Widget}, text::Text,
};

use super::{
    component::*,
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
            _ => {}
        }
    }
}

impl Component for TextInput {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match key_event.code {
            KeyCode::Esc => {
                self.text = String::new();
                Ok(ComponentStatus::Escaped)
            }
            KeyCode::Enter => Ok(ComponentStatus::Closed),
            _ => {
                self.text_editing_key_event(key_event);
                Ok(ComponentStatus::Active)
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        Paragraph::new(self.get_text_for_display())
            .alignment(Alignment::Left)
            .style(default_style())
            .render(area, buf);
        if self.cursor_position < area.width as usize {
            Some(CursorPosition { x: area.x + self.cursor_position as u16, y: area.y })
        } else {
            None
        }
    }
}
