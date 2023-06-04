use anyhow::Result;
use crossterm::event::{KeyEvent, KeyCode};
use tui::{widgets::{Paragraph, Widget}, layout::{Alignment, Rect}, buffer::Buffer, text::{Span, Line}, style::Modifier};

use super::util::{Component, default_style, ComponentStatus};


pub struct TextInput {
    pub active: bool,
    masked: bool,
    max_length: usize,
    pub text_entered: String,
}

impl TextInput {
    pub fn new(max_length: usize, masked: bool) -> Self {
        Self {
            active: true,
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
        let mut text = vec![
            Span::styled(self.get_text_for_display(), default_style()),
        ];
        if self.active {
            text.push(Span::styled("█", default_style().add_modifier(Modifier::RAPID_BLINK)))
        }
        Paragraph::new(Line::from(text))
            .alignment(Alignment::Left)
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

