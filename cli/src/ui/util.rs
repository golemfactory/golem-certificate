use std::{
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

use anyhow::Result;
use crossterm::event::KeyEvent;
use serde::Serialize;
use tui::{
    buffer::Buffer,
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Style},
};

#[derive(PartialEq)]
pub enum ComponentStatus {
    Active,
    Closed,
    Escaped,
}

pub trait Component {
    fn render(&mut self, area: Rect, buf: &mut Buffer);
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus>;
}

pub type Height = u16;
pub type Width = u16;

pub trait SizedComponent: Component {
    fn get_render_size(&self, area: Rect) -> (Height, Width);
}

pub type CalculateHeight = Box<dyn Fn(u16) -> u16>;
pub type CalculateWidth = Box<dyn Fn(u16) -> u16>;
pub type AreaCalculators = (CalculateHeight, CalculateWidth);

#[allow(dead_code)]
pub fn identity_area_calculators() -> AreaCalculators {
    (Box::new(|n: u16| n), Box::new(|n: u16| n))
}

pub fn default_style() -> Style {
    Style::default().fg(Color::Cyan).bg(Color::Black)
}

pub fn get_middle_rectangle(area: Rect, height: u16, width: u16) -> Rect {
    let horizontal_border = area.height.saturating_sub(height) / 2;
    let vertical_border = area.width.saturating_sub(width) / 2;
    let row = Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints(vec![
            Constraint::Max(horizontal_border),
            Constraint::Min(height),
            Constraint::Max(horizontal_border),
        ])
        .split(area)[1];
    let message_box = Layout::default()
        .direction(layout::Direction::Horizontal)
        .constraints(vec![
            Constraint::Max(vertical_border),
            Constraint::Min(width),
            Constraint::Max(vertical_border),
        ])
        .split(row)[1];
    message_box
}

pub fn save_json_to_file<C: ?Sized + Serialize>(
    path: impl AsRef<Path>,
    content: &C,
) -> io::Result<()> {
    let mut writer = BufWriter::new(fs::File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, content)?;
    let _ = writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}
