use std::fmt::Write;

use anyhow::Result;
use crossterm::event::KeyEvent;
use golem_certificate::{create_key_pair, KeyPair};
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, BorderType, Borders, Padding, Widget},
};

use crate::utils::save_json_to_file;

use super::{
    component::*,
    modal::{ModalMessage, ModalMultipleChoice},
    multiple_choice::OVERWRITE_CHOICES,
    save_file_dialog::SaveFileDialog,
    util::default_style,
};

pub struct CreateKeyPairDialog {
    keypair: KeyPair,
    save_file_dialog: SaveFileDialog,
    overwrite_dialog: Option<ModalMultipleChoice>,
    save_error: Option<ModalMessage>,
    saved_message: Option<ModalMessage>,
}

impl CreateKeyPairDialog {
    pub fn new() -> Result<Self> {
        let dialog = Self {
            keypair: create_key_pair(),
            save_file_dialog: SaveFileDialog::new()?,
            overwrite_dialog: None,
            save_error: None,
            saved_message: None,
        };
        Ok(dialog)
    }

    fn save_keypair(&mut self, overwrite: bool) {
        let path = self.save_file_dialog.save_path.as_ref().unwrap().clone();
        let mut file_exists_message = String::new();
        let mut create_key_path = |extension: &str| {
            let mut key_path = path.clone();
            key_path.set_extension(extension);
            if key_path.exists() && !overwrite {
                writeln!(
                    &mut file_exists_message,
                    "File exists: {}",
                    key_path.to_string_lossy()
                )
                .unwrap();
            }
            key_path
        };
        let public_key_path = create_key_path("pub.json");
        let private_key_path = create_key_path("key.json");
        if !file_exists_message.is_empty() {
            let dialog =
                ModalMultipleChoice::new("File exists", file_exists_message, OVERWRITE_CHOICES, 1);
            self.overwrite_dialog = Some(dialog);
        } else {
            let result = save_json_to_file(&private_key_path, &self.keypair.private_key)
                .and_then(|_| save_json_to_file(&public_key_path, &self.keypair.public_key));
            match result {
                Ok(_) => {
                    let message = format!(
                        "Files saved successfully\n{}\n{}\n",
                        private_key_path.to_string_lossy(),
                        public_key_path.to_string_lossy()
                    );
                    let dialog = ModalMessage::new("Keypair saved", message);
                    self.saved_message = Some(dialog);
                }
                Err(err) => {
                    let dialog = ModalMessage::new("Error saving file", err.to_string());
                    self.save_error = Some(dialog);
                }
            }
        }
    }
}

impl Component for CreateKeyPairDialog {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(modal) = self.saved_message.as_mut() {
            modal.handle_key_event(key_event)
        } else if let Some(modal) = self.save_error.as_mut() {
            match modal.handle_key_event(key_event)? {
                ComponentStatus::Active => (),
                _ => {
                    self.save_error = None;
                }
            }
            Ok(ComponentStatus::Active)
        } else if let Some(multiple_choice) = self.overwrite_dialog.as_mut() {
            match multiple_choice.handle_key_event(key_event)? {
                ComponentStatus::Active => (),
                ComponentStatus::Closed => {
                    if multiple_choice.get_selected() == OVERWRITE_CHOICES[0] {
                        self.save_keypair(true);
                    }
                    self.overwrite_dialog = None;
                }
                ComponentStatus::Escaped => self.overwrite_dialog = None,
            }
            Ok(ComponentStatus::Active)
        } else {
            match self.save_file_dialog.handle_key_event(key_event)? {
                ComponentStatus::Closed => {
                    self.save_keypair(false);
                    Ok(ComponentStatus::Active)
                }
                status => Ok(status),
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let block = Block::default()
            .title("Save keypair")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(default_style())
            .padding(Padding::vertical(1));
        let dialog_inner = block.inner(area);
        block.render(area, buf);

        let cursor = self.save_file_dialog.render(dialog_inner, buf);

        if let Some(modal) = self.save_error.as_mut() {
            modal.render(dialog_inner, buf)
        } else if let Some(modal) = self.overwrite_dialog.as_mut() {
            modal.render(dialog_inner, buf)
        } else if let Some(modal) = self.saved_message.as_mut() {
            modal.render(dialog_inner, buf)
        } else {
            cursor
        }
    }
}
