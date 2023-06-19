use super::{
    component::*,
    editors::*,
    modal::{ModalMessage, ModalMultipleChoice, ModalWindow, ModalWithComponent},
    multiple_choice::{MultipleChoice, EXIT_WITHOUT_SAVE, OVERWRITE_CHOICES, SIGN_OR_TEMPLATE},
    open_file_dialog::OpenFileDialog,
    save_file_dialog::SaveFileDialog,
    util::{default_style, read_json_file, reduce_area_fixed, save_json_to_file},
};

use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::KeyEvent;
use gcert::SignatureAlgorithm;
use golem_certificate::{self as gcert, sign_json, Signer};
use serde::Serialize;
use serde_json::Value;
use tui::{
    layout::Rect,
    widgets::{Block, BorderType, Borders, Padding, Widget},
};

const DEFAULT_OR_TEMPLATE: [&str; 2] = ["Default values", "Load template"];

pub trait DocumentEditor {
    fn allow_self_sign(&self) -> bool;
    fn get_document_type(&self) -> &'static str;
    fn editors_mut(&mut self) -> Vec<&mut dyn EditorComponent>;
    fn load_template(&mut self, template: Value);
    fn get_document(&self) -> Result<Value>;
    fn get_template_json(&self) -> Value;
    fn create_signed_document(
        &self,
        algorithm: SignatureAlgorithm,
        signature_value: Vec<u8>,
        signer: Signer,
    ) -> serde_json::Result<Value>;
    fn validate_signed_document(&self, signed_document: Value) -> gcert::Result<Value>;
}

pub struct SignedDocumentEditor {
    active_editor_idx: usize,
    document_editor: Box<dyn DocumentEditor>,
    sign_or_template: MultipleChoice,
    signature_editor: Option<(ModalWindow, SignatureEditor)>,
    save_file_dialog: Option<ModalWithComponent<SaveFileDialog>>,
    popup: Option<ModalMultipleChoice>,
    error: Option<ModalMessage>,
    confirmation: Option<ModalMessage>,
    load_template_dialog: Option<ModalWithComponent<OpenFileDialog>>,
    initialized: bool,
}

impl SignedDocumentEditor {
    pub fn new(document_editor: Box<dyn DocumentEditor>) -> Self {
        let mut sign_or_template = MultipleChoice::new(SIGN_OR_TEMPLATE, 0);
        sign_or_template.active = false;
        let start_from_template = ModalMultipleChoice::new(
            format!("{} editor", document_editor.get_document_type()),
            "Start with default value?",
            DEFAULT_OR_TEMPLATE,
            0,
        );

        Self {
            active_editor_idx: 0,
            document_editor,
            sign_or_template,
            signature_editor: None,
            save_file_dialog: None,
            popup: Some(start_from_template),
            error: None,
            confirmation: None,
            load_template_dialog: None,
            initialized: false,
        }
    }

    fn initialize(&mut self) {
        self.init();
        self.initialized = true;
        self.load_template_dialog = None;
        self.popup = None;
    }

    fn add_signature_editor(&mut self) {
        let mut signature_editor = SignatureEditor::new(self.document_editor.allow_self_sign());
        signature_editor.init();
        let modal_window = ModalWindow::new("Signature details");
        self.signature_editor = Some((modal_window, signature_editor));
    }

    fn open_save_dialog(&mut self) -> Result<()> {
        let save_file_dialog = SaveFileDialog::new()?;
        let area_calculators = reduce_area_fixed(4, 4);
        let title = if self.sign_or_template.get_selected() == SIGN_OR_TEMPLATE[0] {
            format!("Save {}", self.document_editor.get_document_type())
        } else {
            "Save template".into()
        };
        self.save_file_dialog = Some(ModalWithComponent::new(
            title,
            save_file_dialog,
            area_calculators,
        ));
        Ok(())
    }

    fn create_and_validate_signature(&mut self) -> Option<Value> {
        let value = self.document_editor.get_document().unwrap();
        let (key, signer) = self
            .signature_editor
            .as_ref()
            .unwrap()
            .1
            .get_signing_key_and_signer()
            .unwrap();
        match sign_json(&value, &key) {
            Ok((algorithm, signature)) => {
                let signed_document = self
                    .document_editor
                    .create_signed_document(algorithm, signature, signer);
                match signed_document {
                    Ok(value) => match self.document_editor.validate_signed_document(value) {
                        Ok(validated_value) => Some(validated_value),
                        Err(err) => {
                            let title = format!(
                                "Validation error on signed {}",
                                self.document_editor.get_document_type()
                            );
                            let error = ModalMessage::new(title, err.to_string());
                            self.error = Some(error);
                            None
                        }
                    },
                    Err(err) => {
                        let title = format!(
                            "Error serializing signed {}",
                            self.document_editor.get_document_type()
                        );
                        let error = ModalMessage::new(title, err.to_string());
                        self.error = Some(error);
                        None
                    }
                }
            }
            Err(err) => {
                let title = format!("Error signing {}", self.document_editor.get_document_type());
                let error = ModalMessage::new(title, err.to_string());
                self.error = Some(error);
                None
            }
        }
    }

    fn save_file(&mut self, overwrite: bool) {
        let path = self
            .save_file_dialog
            .as_ref()
            .unwrap()
            .get_component()
            .save_path
            .as_ref()
            .unwrap()
            .clone();
        if path.exists() && !overwrite {
            let multiple_choice = ModalMultipleChoice::new(
                "File exists",
                format!("{}", path.to_string_lossy()),
                OVERWRITE_CHOICES,
                1,
            );
            self.popup = Some(multiple_choice);
        } else if self.sign_or_template.get_selected() == SIGN_OR_TEMPLATE[0] {
            if let Some(value) = self.create_and_validate_signature() {
                self.save_json(&path, &value);
            }
        } else {
            let value = self.document_editor.get_template_json();
            self.save_json(&path, &value);
        }
    }

    fn save_json<C: Serialize>(&mut self, path: &PathBuf, content: &C) {
        match save_json_to_file(path, content) {
            Ok(_) => {
                let message = format!("File saved successfully\n{}", path.to_string_lossy());
                let dialog = ModalMessage::new("File saved", message);
                self.confirmation = Some(dialog);
            }
            Err(err) => {
                let error = ModalMessage::new("Error saving file", err.to_string());
                self.error = Some(error);
            }
        }
    }
}

impl EditorGroup for SignedDocumentEditor {
    fn editor_group_state_mut(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>) {
        let mut editors = self.document_editor.editors_mut();
        editors.push(&mut self.sign_or_template);
        (&mut self.active_editor_idx, editors)
    }
}

impl Component for SignedDocumentEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(confirmation) = self.confirmation.as_mut() {
            confirmation.handle_key_event(key_event)
        } else if let Some(error) = self.error.as_mut() {
            match error.handle_key_event(key_event)? {
                ComponentStatus::Active => (),
                _ => self.error = None,
            }
            Ok(ComponentStatus::Active)
        } else if let Some(load_template_dialog) = self.load_template_dialog.as_mut() {
            match load_template_dialog.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => (),
                    ComponentStatus::Escaped => self.load_template_dialog = None,
                    ComponentStatus::Closed => {
                        let path = load_template_dialog
                            .get_component()
                            .selected
                            .as_ref()
                            .unwrap();
                        match read_json_file(path) {
                            Ok(value) => {
                                self.document_editor.load_template(value);
                                self.initialize();
                            }
                            Err(err) => {
                                self.error = Some(ModalMessage::new("Error loading template", err));
                            }
                        }
                    }
                },
                Err(err) => {
                    self.error = Some(ModalMessage::new(
                        "Cannot perform operation",
                        err.to_string(),
                    ))
                }
            }
            Ok(ComponentStatus::Active)
        } else if let Some(popup) = self.popup.as_mut() {
            match popup.handle_key_event(key_event)? {
                ComponentStatus::Active => Ok(ComponentStatus::Active),
                ComponentStatus::Escaped => {
                    self.popup = None;
                    if self.initialized {
                        Ok(ComponentStatus::Active)
                    } else {
                        Ok(ComponentStatus::Escaped)
                    }
                }
                ComponentStatus::Closed => {
                    let selected = popup.get_selected();
                    if selected == EXIT_WITHOUT_SAVE[0] {
                        self.popup = None;
                        Ok(ComponentStatus::Escaped)
                    } else if selected == OVERWRITE_CHOICES[0] {
                        self.popup = None;
                        self.save_file(true);
                        Ok(ComponentStatus::Active)
                    } else if selected == DEFAULT_OR_TEMPLATE[0] {
                        self.popup = None;
                        self.initialize();
                        Ok(ComponentStatus::Active)
                    } else if selected == DEFAULT_OR_TEMPLATE[1] {
                        match OpenFileDialog::new() {
                            Ok(dialog) => {
                                self.load_template_dialog = Some(ModalWithComponent::new(
                                    "Load template",
                                    dialog,
                                    reduce_area_fixed(4, 4),
                                ))
                            }
                            Err(err) => {
                                self.error = Some(ModalMessage::new(
                                    "Error creating open file dialog",
                                    err.to_string(),
                                ))
                            }
                        }
                        Ok(ComponentStatus::Active)
                    } else {
                        self.popup = None;
                        Ok(ComponentStatus::Active)
                    }
                }
            }
        } else if let Some(save_file_dialog) = self.save_file_dialog.as_mut() {
            match save_file_dialog.handle_key_event(key_event)? {
                ComponentStatus::Active => (),
                ComponentStatus::Escaped => self.save_file_dialog = None,
                ComponentStatus::Closed => self.save_file(false),
            }
            Ok(ComponentStatus::Active)
        } else if let Some((_, signature_editor)) = self.signature_editor.as_mut() {
            let editor: &mut dyn EditorGroup = signature_editor;
            match editor.handle_key_event(key_event)? {
                ComponentStatus::Active => Ok(ComponentStatus::Active),
                ComponentStatus::Escaped => {
                    self.signature_editor = None;
                    Ok(ComponentStatus::Active)
                }
                ComponentStatus::Closed => {
                    match signature_editor.get_signing_key_and_signer() {
                        Some(_) => {
                            if self.create_and_validate_signature().is_some() {
                                self.open_save_dialog()?;
                            }
                        }
                        None => {
                            self.error = Some(ModalMessage::new(
                                "Error",
                                "Missing signing key or certificate.",
                            ))
                        }
                    }
                    Ok(ComponentStatus::Active)
                }
            }
        } else {
            let editor_group: &mut dyn EditorGroup = self;
            match editor_group.handle_key_event(key_event)? {
                ComponentStatus::Active => (),
                ComponentStatus::Escaped => {
                    self.popup = Some(ModalMultipleChoice::new(
                        "Exit without saving?",
                        "Changes will be lost.",
                        EXIT_WITHOUT_SAVE,
                        1,
                    ));
                }
                ComponentStatus::Closed => {
                    if self.sign_or_template.get_selected() == SIGN_OR_TEMPLATE[0] {
                        match self.document_editor.get_document() {
                            Ok(_) => {
                                self.add_signature_editor();
                            }
                            Err(err) => {
                                let title = format!(
                                    "Incomplete {}",
                                    self.document_editor.get_document_type()
                                );
                                self.error = Some(ModalMessage::new(title, err.to_string()));
                            }
                        }
                    } else {
                        self.open_save_dialog()?;
                    }
                }
            }
            Ok(ComponentStatus::Active)
        }
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        let title = format!("{} editor", self.document_editor.get_document_type());
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(default_style())
            .padding(Padding::uniform(1));

        let editor_area = block.inner(area);
        block.render(area, buf);

        let editor_group: &mut dyn EditorGroup = self;
        let mut cursor = editor_group.render(editor_area, buf);
        if let Some((modal_window, signature_editor)) = self.signature_editor.as_mut() {
            let inner_area = modal_window.render(
                editor_area,
                buf,
                area.height.saturating_sub(4),
                area.width.saturating_sub(4),
            );
            let editor: &mut dyn EditorGroup = signature_editor;
            cursor = editor.render(inner_area, buf);
        }
        if let Some(save_file_dialog) = self.save_file_dialog.as_mut() {
            cursor = save_file_dialog.render(editor_area, buf);
        }
        if let Some(popup) = self.popup.as_mut() {
            cursor = popup.render(editor_area, buf);
        }
        if let Some(load_template_dialog) = self.load_template_dialog.as_mut() {
            cursor = load_template_dialog.render(editor_area, buf);
        }
        if let Some(error) = self.error.as_mut() {
            cursor = error.render(editor_area, buf);
        }
        if let Some(confirmation) = self.confirmation.as_mut() {
            cursor = confirmation.render(editor_area, buf);
        }
        cursor
    }
}
