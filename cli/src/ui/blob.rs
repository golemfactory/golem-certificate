use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::layout::Direction;
use tui::style::Modifier;
use tui::widgets::{Clear, Padding};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

use super::util::{Component, default_style, get_middle_rectangle};

trait EventHandler {
    fn handle_key_event(&mut self, key_event: KeyEvent);
}


struct QuitState {
    quitting: bool,
    keep_running: bool,
    exit_modal: bool,
}

impl Default for QuitState {
    fn default() -> Self {
        Self { quitting: false, keep_running: true, exit_modal: false }
    }
}

impl EventHandler for QuitState {
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.exit_modal = true,
            KeyCode::Left => self.quitting = true,
            KeyCode::Right => self.quitting = false,
            KeyCode::Enter => if self.quitting {
                self.keep_running = false;
            } else {
                self.exit_modal = true;
            },
            _ => {},
        }
    }
}

struct Quit {}

impl StatefulWidget for Quit {
    type State = QuitState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let message_box = get_middle_rectangle(area, 7, 19);
        Clear.render(message_box, buf);
        let message_block = Block::default()
            .title("Quit")
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(default_style());

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Max(1),
                Constraint::Min(1),
            ])
            .split(message_block.inner(message_box));
        message_block.render(message_box, buf);

        Paragraph::new("Are you sure?")
            .style(default_style())
            .alignment(Alignment::Center)
            .render(rows[1], buf);

        let choices = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Percentage(50),
                Constraint::Percentage(50),
            ])
            .split(rows[3]);

        let normal = default_style();
        let selected = default_style().add_modifier(Modifier::REVERSED);

        Paragraph::new("Quit")
            .style(if state.quitting { selected } else { normal })
            .alignment(Alignment::Center)
            .render(choices[0], buf);

        Paragraph::new("Cancel")
            .style(if state.quitting { normal } else { selected })
            .alignment(Alignment::Center)
            .render(choices[1], buf);
    }
}
