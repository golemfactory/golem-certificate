use std::fmt::Display;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget},
};

use super::{
    component::*,
    util::{default_style, get_middle_rectangle, highlight_style},
};

struct ModalWindow {
    title: String,
}

impl ModalWindow {
    fn render(&self, area: Rect, buf: &mut Buffer, inner_height: u16, inner_width: u16) -> Rect {
        let window = get_middle_rectangle(area, inner_height + 2, inner_width + 2);
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
}

pub struct ModalMessage {
    height: u16,
    width: u16,
    modal_window: ModalWindow,
    message: String,
}

impl ModalMessage {
    pub fn new<S1: Into<String>, S2: Into<String>>(title: S1, message: S2) -> Self {
        let message: String = message.into();
        let (message_height, message_width) = message_dimensions(&message);
        let title: String = title.into();
        let width = message_width.max(title.len() as u16) + 2;
        let modal_window = ModalWindow { title: title };
        Self {
            height: message_height + 2,
            width,
            modal_window,
            message,
        }
    }
}

impl Component for ModalMessage {
    fn handle_key_event(&mut self, _key_event: KeyEvent) -> Result<ComponentStatus> {
        Ok(ComponentStatus::Closed)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let message_area = self.modal_window.render(area, buf, self.height, self.width);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(1), Constraint::Min(0)])
            .split(message_area);
        let message_top: String = self.message.lines().take(1).collect();
        Paragraph::new(message_top)
            .alignment(Alignment::Center)
            .style(default_style())
            .render(chunks[0], buf);
        let message_rest: String = self.message.lines().skip(1).collect::<Vec<_>>().join("\n");
        Paragraph::new(message_rest)
            .alignment(Alignment::Left)
            .style(default_style())
            .render(chunks[1], buf);
        None
    }
}

pub struct ModalMultipleChoice {
    modal_window: ModalWindow,
    message: String,
    choices: Vec<String>,
    pub selected: usize,
    height: u16,
    width: u16,
}

impl ModalMultipleChoice {
    pub fn new<S1, S2, S3, I>(title: S1, message: S2, choices: I, selected: usize) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Display,
        I: IntoIterator<Item = S3>,
    {
        let message: String = message.into();
        let choices: Vec<String> = choices.into_iter().map(|c| format!(" {} ", c)).collect();
        let (message_height, message_width) = message_dimensions(&message);
        let choices_width = choices.iter().map(|c| c.len() + 2).sum::<usize>() as u16;
        let modal_window = ModalWindow {
            title: title.into(),
        };
        Self {
            modal_window,
            message,
            choices,
            selected,
            height: message_height + 4,
            width: choices_width.max(message_width + 2),
        }
    }

    fn selection_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => self.selected = self.selected.saturating_sub(1),
            KeyCode::Right => {
                if self.selected < self.choices.len() - 1 {
                    self.selected += 1;
                }
            }
            _ => {},
        }
    }
}

impl Component for ModalMultipleChoice {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let res = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            KeyCode::Enter => ComponentStatus::Closed,
            _ => {
                self.selection_key_event(key_event);
                ComponentStatus::Active
            }
        };
        Ok(res)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let message_area = self.modal_window.render(area, buf, self.height, self.width);
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
        styles[self.selected] = highlight_style();

        self.choices
            .iter()
            .zip(choice_areas.into_iter())
            .zip(styles.into_iter())
            .for_each(|((choice, &area), style)| {
                Paragraph::new(choice.clone())
                    .alignment(Alignment::Center)
                    .style(style)
                    .render(area, buf);
            });
        None
    }
}

fn message_dimensions(message: &str) -> (Height, Width) {
    let height = message.lines().count();
    let width = message.lines().map(|l| l.len()).max().unwrap_or(0);
    (height as u16, width as u16)
}

pub struct ModalWithComponent {
    modal: ModalWindow,
    component: Box<dyn SizedComponent>,
}

impl ModalWithComponent {
    pub fn new<S1: Into<String>>(title: S1, component: Box<dyn SizedComponent>) -> Self {
        let modal = ModalWindow {
            title: title.into(),
        };
        Self { modal, component }
    }
}

impl Component for ModalWithComponent {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        self.component.handle_key_event(key_event)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let (height, width) = self.component.get_render_size(area);
        let inner_area = self.modal.render(area, buf, height, width);
        self.component.render(inner_area, buf);
        None
    }
}
