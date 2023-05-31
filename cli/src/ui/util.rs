use anyhow::Result;
use crossterm::event::KeyEvent;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
};

#[derive(PartialEq)]
pub enum ComponentStatus {
    Active, Closed, Escaped
}

pub trait Component {
    fn render(&mut self, area: Rect, buf: &mut Buffer);
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus>;
}

pub fn default_style() -> Style {
    Style::default().fg(Color::Cyan).bg(Color::Black)
}

pub fn get_middle_rectangle(area: Rect, height: u16, width: u16) -> Rect {
    let horizontal_border = (area.height - height) / 2;
    let vertical_border = (area.width - width) / 2;
    let row = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Max(horizontal_border),
            Constraint::Min(height),
            Constraint::Max(horizontal_border),
        ])
        .split(area)[1];
    let message_box = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Max(vertical_border),
            Constraint::Min(width),
            Constraint::Max(vertical_border),
        ])
        .split(row)[1];
    message_box
}
