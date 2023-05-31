use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::layout::Direction;
use tui::style::Modifier;
use tui::text::Span;
use tui::widgets::{ListItem, List, ListState};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Borders, Paragraph, StatefulWidget, Widget},
};

use super::util::{Component, default_style};

#[derive(Default)]
pub struct OpenFileDialog {
    current_directory: PathBuf,
    file_names: Vec<std::ffi::OsString>,
    list_state: ListState,
}

impl OpenFileDialog {
    pub fn new() -> Result<Self> {
        let mut dialog = OpenFileDialog::default();
        let current_directory = std::env::current_dir()?;
        dialog.set_directory(current_directory)?;
        Ok(dialog)
    }

    fn go_to_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.current_directory.parent() {
            let directory_name = self.current_directory.file_name()
                .map(|filename| filename.to_owned())
                .ok_or_else(|| anyhow::anyhow!("Some error happened reading the filename of current directory"))?;
            self.set_directory(parent.to_path_buf())?;
            let previous_directory_index = self.file_names.iter().enumerate()
                .find_map(|(idx, file_name)| if *file_name == directory_name { Some(idx) } else { None });
            self.list_state = ListState::default().with_selected(previous_directory_index.or(Some(0)));
        }
        Ok(())
    }

    fn set_directory(&mut self, directory: PathBuf) -> Result<()> {
        self.current_directory = directory.canonicalize()?;
        self.file_names = fs::read_dir(directory)?
            .map(|res| res.map(|entry| entry.file_name()))
            .collect::<Result<Vec<_>, _>>()?;
        self.file_names.sort();
        self.file_names.insert(0, std::ffi::OsStr::new("..").into());
        self.list_state = ListState::default().with_selected(Some(0));
        Ok(())
    }
}

impl Component for OpenFileDialog {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::raw(self.current_directory.to_string_lossy()))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(default_style());
        let list_area = block.inner(area);
        block.render(area, buf);

        let list_parts = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Max(1),
                Constraint::Min(1),
                Constraint::Max(1),
            ])
            .split(list_area);

        let list_items = self.file_names.iter()
            .map(|entry| ListItem::new(entry.to_string_lossy()))
            .collect::<Vec<_>>();

        let list = List::new(list_items)
            .style(default_style())
            .highlight_style(default_style().add_modifier(Modifier::REVERSED));
        StatefulWidget::render(list, list_parts[1], buf, &mut self.list_state);
        if self.list_state.offset() > 0 {
            Paragraph::new("  ^^^^^")
                .style(default_style())
                .render(list_parts[0], buf)
        }
        if (self.list_state.offset() + list_parts[1].height as usize) < self.file_names.len() {
            Paragraph::new("  vvvvv")
                .style(default_style())
                .render(list_parts[2], buf)
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Up => {
                if let Some(idx) = self.list_state.selected().and_then(|i| i.checked_sub(1)) {
                    self.list_state.select(Some(idx));
                }
            }
            KeyCode::Down => {
                if let Some(idx) = self.list_state.selected() {
                    if idx < self.file_names.len() - 1 {
                        self.list_state.select(Some(idx + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = self.list_state.selected() {
                    let path = {
                        let mut path = self.current_directory.clone();
                        path.push(&self.file_names[idx]);
                        path.canonicalize()?
                    };
                    if path.is_dir() {
                        self.set_directory(path)?;
                    } else {
                        println!("Selected file {}", path.to_string_lossy());
                    }
                }
            }
            KeyCode::Backspace => {
                self.go_to_parent()?;
            }
            _ => {}
        }
        Ok(())
    }
}
