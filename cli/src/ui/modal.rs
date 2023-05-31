use std::fmt::Display;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::layout::Direction;
use tui::style::Modifier;
use tui::widgets::{Clear, Padding};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

use super::util::{Component, default_style, get_middle_rectangle, ComponentStatus};

struct ModalWindow {
    inner_height: u16,
    inner_width: u16,
    title: String,
}

type Width = usize;
type Height = usize;

impl ModalWindow {
    fn render(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let window = get_middle_rectangle(area, self.inner_height + 2, self.inner_width + 2);
        Clear.render(window, buf);
        let border = Block::default()
            .title(self.title.clone())
            .title_alignment(Alignment::Left)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .padding(Padding::uniform(1))
            .style(default_style());
        let inner_area = border.inner(window);
        border.render(window, buf);
        inner_area
    }

    fn message_dimensions(message: &str) -> (Height, Width) {
        let height = message.lines().count();
        let width = message.lines().map(|l| l.len()).max().unwrap_or(0);
        (height, width)
    }
}

pub struct ModalMessage {
    modal_window: ModalWindow,
    message: String,
}

impl ModalMessage {
    pub fn new<S1: Into<String>, S2: Into<String>>(title: S1, message: S2) -> Self {
        let message: String = message.into();
        let (message_height, message_width) = ModalWindow::message_dimensions(&message);
        let modal_window = ModalWindow {
            inner_height: message_height as u16 + 2,
            inner_width: message_width as u16 + 2,
            title: title.into(),
        };
        Self { modal_window, message }
    }
}

impl Component for ModalMessage {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let message_area = self.modal_window.render(area, buf);
        Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .style(default_style())
            .render(message_area, buf);
    }

    fn handle_key_event(&mut self, _key_event: KeyEvent) -> Result<ComponentStatus> {
        Ok(ComponentStatus::Closed)
    }
}

pub struct ModalMultipleChoice {
    modal_window: ModalWindow,
    message: String,
    choices: Vec<String>,
    selected: usize,
}

impl ModalMultipleChoice {
    pub fn new<S1, S2, S3, I>(title: S1, message: S2, choices: I, selected: usize) -> Self
    where
        S1: Into<String>, S2: Into<String>, S3: Display,
        I: IntoIterator<Item = S3>
    {
        let message: String = message.into();
        let choices: Vec<String> = choices.into_iter().map(|c| format!(" {} ", c)).collect();
        let (message_height, message_width) = ModalWindow::message_dimensions(&message);
        let choices_width: usize = choices.iter().map(|c| c.len() + 2).sum();
        let modal_window = ModalWindow {
            inner_height: message_height as u16 + 4,
            inner_width: std::cmp::max(message_width + 2, choices_width) as u16,
            title: title.into(),
        };
        Self { modal_window, message, choices, selected }
    }
}

impl Component for ModalMultipleChoice {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let message_area = self.modal_window.render(area, buf);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(1),
                Constraint::Max(1),
                Constraint::Max(1),
            ])
            .split(message_area);

        Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .style(default_style())
            .render(rows[0], buf);

        let choice_constraints =
            vec![Constraint::Ratio(1, self.choices.len() as u32); self.choices.len()];

        let choice_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(choice_constraints)
            .split(rows[2]);
        let mut styles = vec![default_style(); self.choices.len()];
        styles[self.selected] = default_style().add_modifier(Modifier::REVERSED);

        self.choices.iter()
            .zip(choice_areas.into_iter())
            .zip(styles.into_iter())
            .for_each(|((choice, &area), style)| {
                Paragraph::new(choice.clone())
                    .alignment(Alignment::Center)
                    .style(style)
                    .render(area, buf);
            });
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let res = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            KeyCode::Enter => ComponentStatus::Closed,
            KeyCode::Left => {
                if let Some(selected) = self.selected.checked_sub(1) {
                    self.selected = selected;
                }
                ComponentStatus::Active
            }
            KeyCode::Right => {
                if self.selected < self.choices.len() - 1 {
                    self.selected += 1;
                }
                ComponentStatus::Active
            }
            _ => ComponentStatus::Active
        };
        Ok(res)
    }
}
