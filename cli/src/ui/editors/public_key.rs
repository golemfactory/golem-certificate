use super::*;

use std::{fs, path::PathBuf};

use anyhow::Result;
use golem_certificate::Key;

use crate::ui::{open_file_dialog::OpenFileDialog, modal::ModalWindow};

struct KeyFile {
    filename: String,
    key: Key,
}

struct ModalOpenFileDialog {
    modal: ModalWindow,
    dialog: OpenFileDialog,
}

#[derive(Default)]
pub struct PublicKeyEditor {
    key: Option<KeyFile>,
    open_file_dialog: Option<ModalOpenFileDialog>,
    error_message: Option<ModalMessage>,
    active: bool,
}

impl PublicKeyEditor {
    pub fn new(key: Key) -> Self {
        let mut editor = Self::default();
        editor.key = Some(KeyFile {
            filename: "Loaded from template".into(),
            key,
        });
        editor
    }

    pub fn get_key(&self) -> Option<Key> {
        self.key.as_ref().map(|key| key.key.clone())
    }
}

impl EditorComponent for PublicKeyEditor {
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
            match open_file_dialog.dialog.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Closed => {
                        if let Some(path) = open_file_dialog.dialog.selected.as_ref() {
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
                        match OpenFileDialog::new() {
                            Ok(open_file_dialog) => {
                                let modal_window = ModalWindow::new("Open public key");
                                self.open_file_dialog = Some(ModalOpenFileDialog {
                                    modal: modal_window,
                                    dialog: open_file_dialog,
                                });
                            },
                            Err(err) => self.error_message = Some(ModalMessage::new("Error opening file dialog", err.to_string())),
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
        writeln!(text, "Public key: {}", key_string).unwrap();
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.get_highlight().map(|_| 0)
    }

    fn render_modal(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let mut cursor = None;
        if let Some(open_file_dialog) = self.open_file_dialog.as_mut() {
            let height = area.height.saturating_sub(6);
            let width = (area.width * 8) / 10;
            let inner_area = open_file_dialog.modal.render(area, buf, height, width);
            cursor = open_file_dialog.dialog.render(inner_area, buf)
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
