use anyhow::Result;
use crossterm::event::KeyEvent;

use tui::{
    buffer::Buffer,
    layout::Rect,
};

#[derive(PartialEq)]
pub enum ComponentStatus {
    Active,
    Closed,
    Escaped,
}

#[derive(Debug)]
pub struct CursorPosition {
    pub x: u16,
    pub y: u16,
}

pub type Cursor = Option<CursorPosition>;

pub trait Component {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus>;
    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor;
}

pub type Height = u16;
pub type Width = u16;

pub trait SizedComponent: Component {
    fn get_render_size(&self, area: Rect) -> (Height, Width);
}
