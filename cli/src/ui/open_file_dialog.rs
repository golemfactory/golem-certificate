use std::cmp::Ordering;
use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::Span,
    widgets::{Block, BorderType, Borders, ListItem, List, ListState, Paragraph, StatefulWidget, Widget},
};

use super::modal::ModalMessage;
use super::util::{Component, default_style, ComponentStatus};

struct FolderEntry {
    filename: std::ffi::OsString,
    directory: bool,
}

pub struct OpenFileDialog {
    pub active: bool,
    pub border_type: BorderType,
    pub current_directory: PathBuf,
    files: Vec<FolderEntry>,
    list_state: ListState,
    error_message: Option<ModalMessage>,
    pub selected: Option<PathBuf>,
}

impl OpenFileDialog {
    pub fn new() -> Result<Self> {
        let mut dialog = Self {
            active: true,
            border_type: BorderType::Rounded,
            current_directory: PathBuf::new(),
            files: vec![],
            list_state: ListState::default(),
            error_message: None,
            selected: None,
        };
        let current_directory = std::env::current_dir()?;
        dialog.set_directory(current_directory)?;
        Ok(dialog)
    }

    pub fn get_selected_filename(&self) -> Option<String> {
        let selected_entry = &self.files[self.list_state.selected().unwrap()];
        if selected_entry.directory {
            None
        } else {
            Some(selected_entry.filename.to_string_lossy().into())
        }
    }

    fn go_to_parent(&mut self) -> Result<()> {
        if let Some(parent) = self.current_directory.parent() {
            let directory_name = self.current_directory.file_name()
                .map(|filename| filename.to_owned())
                .ok_or_else(|| anyhow::anyhow!("Some error happened reading the filename of current directory"))?;
            self.set_directory(parent.to_path_buf())?;
            let previous_directory_index = self.files.iter().enumerate()
                .find_map(|(idx, file)| if file.filename == directory_name { Some(idx) } else { None });
            self.list_state = ListState::default().with_selected(previous_directory_index.or(Some(0)));
        }
        Ok(())
    }

    fn set_directory(&mut self, directory: PathBuf) -> Result<()> {
        self.current_directory = directory.canonicalize()?;
        let mut files = fs::read_dir(directory)?.collect::<Result<Vec<_>, _>>()?;
        files.sort_by(|a, b| {
            if a.path().is_dir() && !b.path().is_dir() {
                Ordering::Less
            } else if !a.path().is_dir() && b.path().is_dir() {
                Ordering::Greater
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });
        self.files = files.into_iter()
            .map(|entry| FolderEntry { filename: entry.file_name(), directory: entry.path().is_dir() })
            .collect();
        self.files.insert(0, FolderEntry { filename: std::ffi::OsStr::new("..").into(), directory: true });
        self.list_state = ListState::default().with_selected(Some(0));
        Ok(())
    }

    fn handle_key_event_self(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match key_event.code {
            KeyCode::Up => {
                let select = self.list_state.selected().map(|i| i.saturating_sub(1));
                self.list_state.select(select);
            }
            KeyCode::Down => {
                if let Some(idx) = self.list_state.selected() {
                    if idx < self.files.len() - 1 {
                        self.list_state.select(Some(idx + 1));
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = self.list_state.selected() {
                    let path = {
                        let mut path = self.current_directory.clone();
                        path.push(&self.files[idx].filename);
                        path.canonicalize()?
                    };
                    if path.is_dir() {
                        self.set_directory(path)?;
                    } else if path.is_file() {
                        self.selected = Some(path);
                        return Ok(ComponentStatus::Closed);
                    } else {
                        let message = format!("Not a directory neither a file...\n{}", self.files[idx].filename.to_string_lossy());
                        let modal = ModalMessage::new("Error", message);
                        self.error_message = Some(modal);
                    }
                }
            }
            KeyCode::Backspace => {
                self.go_to_parent()?;
            }
            KeyCode::Esc => {
                return Ok(ComponentStatus::Escaped);
            }
            _ => {}
        }
        Ok(ComponentStatus::Active)
    }
}

impl Component for OpenFileDialog {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(Span::raw(self.current_directory.to_string_lossy()))
            .borders(Borders::ALL)
            .border_type(self.border_type)
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

        let list_items = self.files.iter()
            .map(|entry| ListItem::new(format!("{} {}", if entry.directory { "\u{1F4C1}" } else { " " }, entry.filename.to_string_lossy())))
            .collect::<Vec<_>>();

        let mut list = List::new(list_items)
            .style(default_style());
        if self.active {
            list = list.highlight_style(default_style().add_modifier(Modifier::REVERSED));
        }
        StatefulWidget::render(list, list_parts[1], buf, &mut self.list_state);
        if self.list_state.offset() > 0 {
            Paragraph::new("  ^^^^^")
                .style(default_style())
                .render(list_parts[0], buf)
        }
        if (self.list_state.offset() + list_parts[1].height as usize) < self.files.len() {
            Paragraph::new("  vvvvv")
                .style(default_style())
                .render(list_parts[2], buf)
        }
        if let Some(component) = &mut self.error_message {
            component.render(area, buf)
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(component) = &mut self.error_message {
            let res = component.handle_key_event(key_event)?;
            if res != ComponentStatus::Active {
                self.error_message = None;
            }
            Ok(ComponentStatus::Active)
        } else {
            self.handle_key_event_self(key_event)
        }
    }
}
