use std::{
    fs,
    io::{self, BufWriter, Write},
    path::Path,
};

use serde::Serialize;
use serde_json::Value;
use tui::{
    layout::{self, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
};

pub type CalculateHeight = Box<dyn Fn(u16) -> u16>;
pub type CalculateWidth = Box<dyn Fn(u16) -> u16>;
pub type AreaCalculators = (CalculateHeight, CalculateWidth);

pub fn reduce_area_fixed(height: u16, width: u16) -> AreaCalculators {
    (Box::new(move |h| h - height), Box::new(move |w| w - width))
}

pub fn default_style() -> Style {
    Style::default().fg(Color::Cyan).bg(Color::Black)
}

pub fn highlight_style() -> Style {
    default_style().add_modifier(Modifier::REVERSED)
}

pub fn get_middle_rectangle(area: Rect, height: u16, width: u16) -> Rect {
    let horizontal_border = area.height.saturating_sub(height) / 2;
    let vertical_border = area.width.saturating_sub(width) / 2;
    let row = Layout::default()
        .direction(layout::Direction::Vertical)
        .constraints([
            Constraint::Max(horizontal_border),
            Constraint::Min(height),
            Constraint::Max(horizontal_border),
        ])
        .split(area)[1];
    Layout::default()
        .direction(layout::Direction::Horizontal)
        .constraints([
            Constraint::Max(vertical_border),
            Constraint::Min(width),
            Constraint::Max(vertical_border),
        ])
        .split(row)[1]
}

pub fn save_json_to_file<C: Serialize>(path: impl AsRef<Path>, content: &C) -> io::Result<()> {
    let mut writer = BufWriter::new(fs::File::create(path)?);
    serde_json::to_writer_pretty(&mut writer, content)?;
    let _ = writer.write(b"\n")?;
    writer.flush()?;
    Ok(())
}

pub fn read_json_file(path: &Path) -> Result<Value, String> {
    fs::read_to_string(path)
        .map_err(|err| format!("Cannot read file ({})", err))
        .and_then(|contents| {
            serde_json::from_str::<Value>(&contents).map_err(|_| "File contents is not JSON".into())
        })
}
