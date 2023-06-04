use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, BorderType, Borders, Widget},
};

use super::{open_file_dialog::OpenFileDialog, text_input::TextInput};
use super::util::{Component, default_style, ComponentStatus};

#[derive(PartialEq)]
enum DialogParts {
    FileBrowser, FilenameInput
}

pub struct SaveFileDialog {
    active_component: DialogParts,
    file_browser: OpenFileDialog,
    filename_input: TextInput,
    pub save_path: Option<PathBuf>,
}

impl SaveFileDialog {
    pub fn new() -> Result<Self> {
        let mut dialog = Self {
            active_component: DialogParts::FileBrowser,
            file_browser: OpenFileDialog::new()?,
            filename_input: TextInput::new(50, false),
            save_path: None,
        };
        dialog.file_browser.border_type = BorderType::Thick;
        Ok(dialog)
    }

    fn file_browser_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = self.file_browser.handle_key_event(key_event)?;
        match status {
            ComponentStatus::Closed => self.save_path = self.file_browser.selected.clone(),
            ComponentStatus::Active =>
                if let Some(filename) = self.file_browser.get_selected_filename() {
                    self.filename_input.text_entered = filename;
                }
            _ => {}
        }
        Ok(status)
    }

    fn filename_input_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = self.filename_input.handle_key_event(key_event)?;
        if status == ComponentStatus::Closed {
            let mut path = self.file_browser.current_directory.clone();
            path.push(&self.filename_input.text_entered);
            self.save_path = Some(path);
        }
        Ok(status)
    }
}

impl Component for SaveFileDialog {
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(3),
                Constraint::Max(3),
            ])
            .split(area);
        self.file_browser.render(chunks[0], buf);

        let block = Block::default()
            .title("Filename")
            .borders(Borders::ALL)
            .border_type(if self.active_component == DialogParts::FilenameInput { BorderType::Thick } else { BorderType::Rounded })
            .style(default_style());
        let filename_input_area = block.inner(chunks[1]);
        block.render(chunks[1], buf);

        self.filename_input.render(filename_input_area, buf);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match key_event.code {
            KeyCode::Tab => {
                self.active_component = match self.active_component {
                    DialogParts::FileBrowser => {
                        self.file_browser.active = false;
                        self.file_browser.border_type = BorderType::Rounded;
                        DialogParts::FilenameInput
                    }
                    DialogParts::FilenameInput => {
                        self.file_browser.active = true;
                        self.file_browser.border_type = BorderType::Thick;
                        DialogParts::FileBrowser
                    }
                };
                Ok(ComponentStatus::Active)
            }
            _ => match self.active_component {
                DialogParts::FileBrowser => self.file_browser_key_event(key_event),
                DialogParts::FilenameInput => self.filename_input_key_event(key_event),
            }
        }
    }
}
