use std::fmt::Display;

use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph, Widget},
};

use super::{
    component::*,
    util::{default_style, get_middle_rectangle}, multiple_choice::MultipleChoice,
};

pub struct ModalWindow {
    title: String,
}

impl ModalWindow {
    pub fn new<S: Into<String>>(title: S) -> Self {
        Self { title: title.into() }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer, inner_height: u16, inner_width: u16) -> Rect {
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
        let modal_window = ModalWindow::new(title);
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
            .constraints([Constraint::Max(1), Constraint::Min(0)])
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
    multiple_choice: MultipleChoice,
    message: String,
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
        let modal_window = ModalWindow::new(title);
        let message: String = message.into();
        let (message_height, message_width) = message_dimensions(&message);
        let choices: Vec<String> = choices.into_iter().map(|c| format!(" {} ", c)).collect();
        let multiple_choice = MultipleChoice::new(choices, selected);
        let choices_width = multiple_choice.get_render_width();
        Self {
            modal_window,
            multiple_choice,
            message,
            height: message_height + 4,
            width: choices_width.max(message_width + 2),
        }
    }

    pub fn get_selected(&self) -> String {
        self.multiple_choice.get_selected()
    }
}

impl Component for ModalMultipleChoice {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        self.multiple_choice.handle_key_event(key_event)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let message_area = self.modal_window.render(area, buf, self.height, self.width);
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Max(1),
                Constraint::Max(1),
            ])
            .split(message_area);

        Paragraph::new(self.message.clone())
            .alignment(Alignment::Center)
            .style(default_style())
            .render(rows[0], buf);

        self.multiple_choice.render(rows[2], buf)
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
        let modal = ModalWindow::new(title);
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
