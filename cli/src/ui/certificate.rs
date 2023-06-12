use std::path::PathBuf;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::{SignedCertificate, schemas::{certificate::Certificate, signature, SIGNED_CERTIFICATE_SCHEMA_ID}, sign_json, Signature, validate_certificate};
use serde::Serialize;
use tui::{layout::Rect, widgets::{StatefulWidget, Block, BorderType, Borders, Widget}};

use super::{
    component::*,
    display_details::certificate_to_string,
    editors::*,
    modal::{ModalMultipleChoice, ModalWindow, ModalMessage, ModalWithComponent},
    multiple_choice::{EXIT_WITHOUT_SAVE, MultipleChoice, SIGN_OR_TEMPLATE, self, OVERWRITE_CHOICES},
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth, identity_area, reduce_area_fixed, save_json_to_file,
    }, save_file_dialog::SaveFileDialog,
};

pub struct SignedCertificateDetails {
    calculate_height: CalculateHeight,
    calculate_width: CalculateWidth,
    render_state: ScrollableTextState,
}

impl SignedCertificateDetails {
    pub fn new(
        cert: &SignedCertificate,
        indent: usize,
        detailed_signer: bool,
        (calculate_height, calculate_width): AreaCalculators,
    ) -> Self {
        let text = certificate_to_string(cert, indent, detailed_signer);
        Self {
            render_state: ScrollableTextState::new(text),
            calculate_height,
            calculate_width,
        }
    }

    pub fn new_with_defaults(cert: &SignedCertificate) -> Self {
        let text = certificate_to_string(cert, 2, false);
        let (calculate_height, calculate_width) = identity_area();
        Self {
            render_state: ScrollableTextState::new(text),
            calculate_height,
            calculate_width,
        }
    }

    pub fn get_text_height(&self) -> usize {
        self.render_state.get_text_height()
    }

    fn scrolling_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_sub(1);
            }
            KeyCode::Down => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_add(1);
            }
            _ => (),
        }
    }
}

impl Component for SignedCertificateDetails {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            _ => {
                self.scrolling_key_event(key_event);
                ComponentStatus::Active
            }
        };
        Ok(status)
    }

    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) -> Cursor {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
        None
    }
}

impl SizedComponent for SignedCertificateDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        (
            (self.calculate_height)(area.height),
            (self.calculate_width)(area.width),
        )
    }
}

#[derive(Default)]
struct CertificateDocumentEditor {
    key_usage_editor: KeyUsageEditor,
    permissions_editor: PermissionsEditor,
    public_key_editor: KeyEditor,
    subject_editor: SubjectEditor,
    validity_period_editor: ValidityPeriodEditor,
}

impl CertificateDocumentEditor {
    fn get_document(&self) -> Result<serde_json::Value> {
        if let Some(key) = self.public_key_editor.get_key() {
            let cert = Certificate {
                key_usage: self.key_usage_editor.get_key_usage(),
                permissions: self.permissions_editor.get_permissions(),
                public_key: key,
                subject: self.subject_editor.get_subject(),
                validity_period: self.validity_period_editor.get_validity_period(),
            };
            Ok(serde_json::to_value(cert)?)
        } else {
            anyhow::bail!("No public key")
        }
    }

    fn editors_mut(&mut self) -> Vec<&mut dyn EditorComponent> {
        vec![
            &mut self.subject_editor,
            &mut self.permissions_editor,
            &mut self.validity_period_editor,
            &mut self.public_key_editor,
            &mut self.key_usage_editor,
        ]
    }
}

pub struct CertificateEditor {
    active_editor_idx: usize,
    document: CertificateDocumentEditor,
    sign_or_template: MultipleChoice,
    signature_editor: Option<(ModalWindow, SignatureEditor)>,
    save_file_dialog: Option<ModalWithComponent<SaveFileDialog>>,
    popup: Option<ModalMultipleChoice>,
    error: Option<ModalMessage>,
    confirmation: Option<ModalMessage>,
}

impl CertificateEditor {
    pub fn new() -> Self {
        let mut sign_or_template = MultipleChoice::new(SIGN_OR_TEMPLATE, 0);
        sign_or_template.active = false;

        let mut editor = Self {
            active_editor_idx: 0,
            document: CertificateDocumentEditor::default(),
            sign_or_template,
            signature_editor: None,
            save_file_dialog: None,
            popup: None,
            error: None,
            confirmation: None,
        };
        editor.init();
        editor
    }

    fn add_signature_editor(&mut self) {
        let mut signature_editor = SignatureEditor::new(true);
        signature_editor.init();
        let modal_window = ModalWindow::new("Signature details");
        self.signature_editor = Some((modal_window, signature_editor));
    }

    fn open_save_dialog(&mut self) -> Result<()> {
        let save_file_dialog = SaveFileDialog::new()?;
        let area_calculators = reduce_area_fixed(4, 4);
        let title = if self.sign_or_template.get_selected() == SIGN_OR_TEMPLATE[0] {
            "Save certificate"
        } else {
            "Save template"
        };
        self.save_file_dialog = Some(ModalWithComponent::new(title, save_file_dialog, area_calculators));
        Ok(())
    }

    fn save_file(&mut self, overwrite: bool) {
        let path = self.save_file_dialog.as_ref().unwrap().get_component().save_path.as_ref().unwrap().clone();
        if path.exists() && !overwrite {
            let multiple_choice = ModalMultipleChoice::new(
                "File exists",
                format!("{}", path.to_string_lossy()),
                &OVERWRITE_CHOICES,
                1,
            );
            self.popup = Some(multiple_choice);
        } else {
            if self.sign_or_template.get_selected() == SIGN_OR_TEMPLATE[0] {
                let value = self.document.get_document().unwrap();
                let (key, signer) = self.signature_editor.as_ref().unwrap().1.get_signing_key_and_signer().unwrap();
                match sign_json(&value, &key) {
                    Ok((algorithm, signature)) => {
                        let signed_cert = SignedCertificate {
                            schema: SIGNED_CERTIFICATE_SCHEMA_ID.to_string(),
                            certificate: value,
                            signature: Box::new(Signature { algorithm, value: signature, signer }),
                        };
                        match serde_json::to_value(signed_cert.clone()) {
                            Ok(value) => match validate_certificate(value, None) {
                                Ok(_) => self.save_json(&path, &signed_cert),
                                Err(err) => {
                                    let error = ModalMessage::new("Validation error on signed certificate", err.to_string());
                                    self.error = Some(error);
                                }
                            }
                            Err(err) => {
                                let error = ModalMessage::new("Error serializing signed certificate", err.to_string());
                                self.error = Some(error);
                            }
                        }
                    }
                    Err(err) => {
                        let error = ModalMessage::new("Error signing certificate", err.to_string());
                        self.error = Some(error);
                    }
                }
            } else {
                let value = self.document.get_document().unwrap();
                self.save_json(&path, &value);
            }
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

impl EditorGroup for CertificateEditor {
    fn editor_group_state_mut(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>) {
        let mut editors = self.document.editors_mut();
        editors.push(&mut self.sign_or_template);
        (&mut self.active_editor_idx, editors)
    }
}

impl Component for CertificateEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(confirmation) = self.confirmation.as_mut() {
            confirmation.handle_key_event(key_event)
        } else if let Some(error) = self.error.as_mut() {
            match error.handle_key_event(key_event)? {
                ComponentStatus::Active => {},
                _ => self.error = None,
            }
            Ok(ComponentStatus::Active)
        } else if let Some(popup) = self.popup.as_mut() {
            match popup.handle_key_event(key_event)? {
                ComponentStatus::Active => Ok(ComponentStatus::Active),
                ComponentStatus::Escaped => {
                    self.popup = None;
                    Ok(ComponentStatus::Active)
                }
                ComponentStatus::Closed => {
                    let selected = popup.get_selected();
                    self.popup = None;
                    if selected == EXIT_WITHOUT_SAVE[0] {
                        Ok(ComponentStatus::Escaped)
                    } else if selected == OVERWRITE_CHOICES[0] {
                        self.save_file(true);
                        Ok(ComponentStatus::Active)
                    } else {
                        Ok(ComponentStatus::Active)
                    }
                }
            }
        } else if let Some(save_file_dialog) = self.save_file_dialog.as_mut() {
            match save_file_dialog.handle_key_event(key_event)? {
                ComponentStatus::Active => {},
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
                        Some(_) => self.open_save_dialog()?,
                        None => self.error = Some(ModalMessage::new(
                            "Error",
                            "Missing signing key or certificate.",
                        )),
                    }
                    Ok(ComponentStatus::Active)
                }
            }
        } else {
            let editor_group: &mut dyn EditorGroup = self;
            match editor_group.handle_key_event(key_event)? {
                ComponentStatus::Active => {},
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
                        match self.document.get_document() {
                            Ok(_) => {
                                self.add_signature_editor();
                            }
                            Err(err) => {
                                self.error = Some(ModalMessage::new("Incomplete certificate", err.to_string()));
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
        let block = Block::default()
            .title("Certificate editor")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(default_style());

        let editor_area = block.inner(area);
        block.render(area, buf);

        let editor_group: &mut dyn EditorGroup = self;
        let mut cursor = editor_group.render(editor_area, buf);
        if let Some((modal_window, signature_editor)) = self.signature_editor.as_mut() {
            let inner_area = modal_window.render(editor_area, buf, area.height.saturating_sub(4), area.width.saturating_sub(4));
            let editor: &mut dyn EditorGroup = signature_editor;
            cursor = editor.render(inner_area, buf);
        }
        if let Some(save_file_dialog) = self.save_file_dialog.as_mut() {
            cursor = save_file_dialog.render(editor_area, buf);
        }
        if let Some(popup) = self.popup.as_mut() {
            cursor = popup.render(editor_area, buf);
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
