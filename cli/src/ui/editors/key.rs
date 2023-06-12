use super::*;

use std::{fs, path::PathBuf};

use anyhow::Result;
use golem_certificate::Key;

use crate::ui::modal::ModalOpenFileDialog;

struct KeyFile {
    filename: String,
    key: Key,
}

pub struct KeyEditor {
    key_type: String,
    key: Option<KeyFile>,
    open_file_dialog: Option<ModalOpenFileDialog>,
    error_message: Option<ModalMessage>,
    active: bool,
}

impl KeyEditor {
    pub fn new<S: Into<String>>(key_type: S, key: Option<Key>) -> Self {
        let mut editor = Self::default();
        editor.key_type = key_type.into();
        editor.key = key.map(|key| KeyFile {
            filename: "Loaded from template".into(),
            key,
        });
        editor
    }

    pub fn get_key(&self) -> Option<Key> {
        self.key.as_ref().map(|key| key.key.clone())
    }
}

impl Default for KeyEditor {
    fn default() -> Self {
        Self {
            key_type: "Public".into(),
            key: None,
            open_file_dialog: None,
            error_message: None,
            active: false,
        }
    }
}

impl EditorComponent for KeyEditor {
    fn enter_from_below(&mut self) {
        self.active = true;
    }

    fn enter_from_top(&mut self) {
        self.active = true;
    }

    fn get_highlight(&self) -> Option<usize> {
        if self.active {
            Some(0)
        } else {
            None
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        if let Some(error_message) = self.error_message.as_mut() {
            match error_message.handle_key_event(key_event) {
                Ok(ComponentStatus::Active) => {},
                _ => self.error_message = None,
            }
            EditorEventResult::KeepActive
        } else if let Some(open_file_dialog) = self.open_file_dialog.as_mut() {
            match open_file_dialog.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Closed => {
                        if let Some(path) = open_file_dialog.get_selected() {
                            match load_key(path) {
                                Ok(key) => {
                                    self.key = Some(key);
                                    self.open_file_dialog = None;
                                }
                                Err(err) =>
                                    self.error_message = Some(ModalMessage::new("Error loading key", err.to_string())),
                            }
                        }
                    },
                    ComponentStatus::Escaped => self.open_file_dialog = None,
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else {
            if self.active {
                match key_event.code {
                    KeyCode::Esc => EditorEventResult::Escaped,
                    KeyCode::Enter => {
                        match ModalOpenFileDialog::new(format!("Open {} key", self.key_type)) {
                            Ok(open_file_dialog) =>
                                self.open_file_dialog = Some(open_file_dialog),
                            Err(err) =>
                                self.error_message = Some(ModalMessage::new("Error opening file dialog", err.to_string())),
                        }
                        EditorEventResult::KeepActive
                    }
                    KeyCode::Down => {
                        self.active = false;
                        EditorEventResult::ExitBottom
                    }
                    KeyCode::Up => {
                        self.active = false;
                        EditorEventResult::ExitTop
                    }
                    _ => EditorEventResult::KeepActive,
                }
            } else {
                EditorEventResult::Inactive
            }
        }
    }

    fn calculate_render_height(&self) -> usize {
        1
    }

    fn get_text_output(&self, text: &mut String) {
        let key_string = match &self.key {
            Some(key) => &key.filename,
            None => "Not loaded",
        };
        writeln!(text, "{} key: {}", self.key_type, key_string).unwrap();
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.get_highlight().map(|_| 0)
    }

    fn render_modal(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let mut cursor = None;
        if let Some(open_file_dialog) = self.open_file_dialog.as_mut() {
            let height = area.height.saturating_sub(6);
            let width = (area.width * 8) / 10;
            cursor = open_file_dialog.render(area, buf, height, width);
        }
        if let Some(error_message) = self.error_message.as_mut() {
            cursor = error_message.render(area, buf)
        }
        cursor
    }
}

fn load_key(path: &PathBuf) -> Result<KeyFile> {
    let key_string = fs::read_to_string(path)?;
    let key: Key = serde_json::from_str(&key_string)?;
    let filename: String = path.file_name()
        .map(|filename| filename.to_string_lossy().into())
        .unwrap_or("Unknown filename".into());
    Ok(KeyFile { filename, key })
}
