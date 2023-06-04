use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use tui::{widgets::{Paragraph, Widget}, layout::{Alignment, Rect}, buffer::Buffer};

use super::util::{Component, default_style, ComponentStatus};


pub struct TextInput {
    masked: bool,
    max_length: usize,
    pub text_entered: String,
}

impl TextInput {
    pub fn new(max_length: usize, masked: bool) -> Self {
        Self {
            masked,
            max_length,
            text_entered: String::new(),
        }
    }

    pub fn get_text_for_display(&self) -> String {
        if self.masked {
            self.text_entered.chars().map(|_| '*').collect()
        } else {
            self.text_entered.clone()
        }
    }
}

impl Component for TextInput {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.get_text_for_display())
            .alignment(Alignment::Left)
            .style(default_style())
            .render(area, buf);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match key_event.code {
            KeyCode::Esc => {
                self.text_entered = String::new();
                Ok(ComponentStatus::Escaped)
            }
            KeyCode::Enter => Ok(ComponentStatus::Closed),
            KeyCode::Char(c) => {
                if self.text_entered.len() < self.max_length {
                    self.text_entered.push(c);
                }
                Ok(ComponentStatus::Active)
            }
            KeyCode::Backspace => {
                self.text_entered.pop();
                Ok(ComponentStatus::Active)
            }
            _ => Ok(ComponentStatus::Active)
        }
    }
}

