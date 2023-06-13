use super::*;

use std::{fs, path::PathBuf};

use anyhow::Result;
use chrono::Utc;
use golem_certificate::{SignedCertificate, validate_certificate_str, Signer, Key};

use crate::ui::{multiple_choice::{MultipleChoice, SIGN_OR_CANCEL}, certificate::SignedCertificateDetails, modal::{ModalMultipleChoice, ModalOpenFileDialog, ModalWithSizedComponent}, util::reduce_area_fixed};

pub struct SignatureEditor {
    active_editor_idx: usize,
    signing_key_editor: KeyEditor,
    signing_certificate_editor: SigningCertificateEditor,
    sign_or_cancel: MultipleChoice,
}

impl SignatureEditor {
    pub fn new(allow_self_sign: bool) -> Self {
        let mut sign_or_cancel = MultipleChoice::new(SIGN_OR_CANCEL, 0);
        sign_or_cancel.active = false;
        Self {
            active_editor_idx: 0,
            signing_key_editor: KeyEditor::new("Signing", None),
            signing_certificate_editor: SigningCertificateEditor::new(allow_self_sign),
            sign_or_cancel,
        }
    }

    pub fn get_signing_key_and_signer(&self) -> Option<(Key, Signer)> {
        let key = self.signing_key_editor.get_key();
        let signer = match self.signing_certificate_editor.signature_type {
            SignatureType::None => None,
            SignatureType::SelfSigned => Some(Signer::SelfSigned),
            SignatureType::Certificate => Some(Signer::Certificate(self.signing_certificate_editor.get_cert().unwrap().to_owned())),
        };
        match (key, signer) {
            (Some(key), Some(signer)) => Some((key, signer)),
            _ => None,
        }
    }
}

impl EditorGroup for SignatureEditor {
    fn editor_group_state_mut(&mut self) -> (&mut usize, Vec<&mut dyn EditorComponent>) {
        let editors: Vec<&mut dyn EditorComponent> = vec![
            &mut self.signing_key_editor,
            &mut self.signing_certificate_editor,
            &mut self.sign_or_cancel,
        ];
        (&mut self.active_editor_idx, editors)
    }
}


#[derive(Default, PartialEq)]
enum SignatureType {
    #[default]
    None,
    SelfSigned,
    Certificate,
}

const SELFSIGNED_OR_CERTIFICATE: &[&str] = &["Self-signed", "Load signing certificate"];

#[derive(Default)]
struct SigningCertificateEditor {
    allow_self_sign: bool,
    highlight: Option<usize>,
    signature_type: SignatureType,
    signed_certificate: Option<(SignedCertificate, String)>,
    signed_certificate_details: Option<ModalWithSizedComponent>,
    signature_type_question: Option<ModalMultipleChoice>,
    open_file_dialog: Option<ModalOpenFileDialog>,
    error: Option<ModalMessage>,
}

impl SigningCertificateEditor {
    fn new(allow_self_sign: bool) -> Self {
        let mut editor = Self::default();
        editor.allow_self_sign = allow_self_sign;
        editor
    }

    fn get_cert(&self) -> Option<&SignedCertificate> {
        self.signed_certificate.as_ref().map(|(cert, _)| cert)
    }

    fn select_signature_type(&mut self) {
        self.signature_type_question = Some(ModalMultipleChoice::new("Signature type", "", SELFSIGNED_OR_CERTIFICATE, 0));
    }

    fn open_certificate_dialog(&mut self) {
        match ModalOpenFileDialog::new("Open signing certificate") {
            Ok(dialog) => self.open_file_dialog = Some(dialog),
            Err(err) => self.error = Some(ModalMessage::new("Error opening 'Open certificate' dialog", &err.to_string())),
        }
    }

    fn open_certificate_details(&mut self) {
        match self.get_cert() {
            Some(cert) => {
                let details = SignedCertificateDetails::new(cert, 2, false, reduce_area_fixed(2, 4));
                let modal = ModalWithSizedComponent::new("Signing certificate details", Box::new(details));
                self.signed_certificate_details = Some(modal);
            },
            None => {},
        }
    }
}

fn read_certificate(path: &PathBuf) -> Result<(SignedCertificate, String), String> {
    fs::read_to_string(path)
        .map_err(|err| format!("Cannot read file\n{}\n{}", path.to_string_lossy(), err.to_string()))
        .and_then(|text| {
            match validate_certificate_str(&text, Some(Utc::now())) {
                Ok(validated_certificate) => {
                    Ok((serde_json::from_str::<SignedCertificate>(&text).unwrap(), validated_certificate.subject.display_name))
                },
                Err(err) => Err(format!("File contents is not valid certificate\n{}\n{}", path.to_string_lossy(), err.to_string()))
            }
        })
}

impl EditorComponent for SigningCertificateEditor {
    fn enter_from_below(&mut self) {
        self.highlight = Some(self.calculate_render_height() - 1);
    }

    fn enter_from_top(&mut self) {
        self.highlight = Some(0);
    }

    fn get_highlight(&self) -> Option<usize> {
        self.highlight
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        if let Some(modal) = self.error.as_mut() {
            match modal.handle_key_event(key_event) {
                Ok(ComponentStatus::Active) => {},
                _ => self.error = None,
            }
            EditorEventResult::KeepActive
        } else if let Some(dialog) = self.open_file_dialog.as_mut() {
            match dialog.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Escaped => self.open_file_dialog = None,
                    ComponentStatus::Closed => {
                        match dialog.get_selected() {
                            Some(path) => {
                                match read_certificate(path) {
                                    Ok(signed_certificate) => {
                                        self.signed_certificate = Some(signed_certificate);
                                        self.signature_type = SignatureType::Certificate;
                                        self.open_file_dialog = None;
                                    },
                                    Err(err) => self.error = Some(ModalMessage::new("Error loading certificate", err)),
                                }
                            },
                            None => self.open_file_dialog = None,
                        }
                    },
                }
                Err(err) => self.error = Some(ModalMessage::new("Error opening certificate", &err.to_string())),
            }
            EditorEventResult::KeepActive
        } else if let Some(type_choice) = self.signature_type_question.as_mut() {
            match type_choice.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Escaped => self.signature_type_question = None,
                    ComponentStatus::Closed => {
                        if type_choice.get_selected() == SELFSIGNED_OR_CERTIFICATE[0] {
                            self.signature_type = SignatureType::SelfSigned;
                        } else {
                            self.open_certificate_dialog();
                        }
                        self.signature_type_question = None;
                    },
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else if let Some(certificate_details) = self.signed_certificate_details.as_mut() {
            match certificate_details.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    _ => self.signed_certificate_details = None,
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else if let Some(highlight) = self.highlight {
            match key_event.code {
                KeyCode::Esc => EditorEventResult::Escaped,
                KeyCode::Down =>
                    if highlight > 0 || self.signature_type != SignatureType::Certificate {
                        self.highlight = None;
                        EditorEventResult::ExitBottom
                    } else {
                        self.highlight = Some(1);
                        EditorEventResult::KeepActive
                    }
                KeyCode::Up =>
                    if highlight == 0 {
                        self.highlight = None;
                        EditorEventResult::ExitTop
                    } else {
                        self.highlight = Some(0);
                        EditorEventResult::KeepActive
                    }
                KeyCode::Enter => {
                    if highlight == 0 {
                        match self.signature_type {
                            SignatureType::None => {
                                if self.allow_self_sign {
                                    self.select_signature_type();
                                    EditorEventResult::KeepActive
                                } else {
                                    self.open_certificate_dialog();
                                    EditorEventResult::KeepActive
                                }
                            }
                            SignatureType::SelfSigned => {
                                self.open_certificate_dialog();
                                EditorEventResult::KeepActive
                            },
                            SignatureType::Certificate => {
                                if self.allow_self_sign {
                                    self.select_signature_type();
                                    EditorEventResult::KeepActive
                                } else {
                                    self.open_certificate_dialog();
                                    EditorEventResult::KeepActive
                                }
                            },
                        }
                    } else {
                        self.open_certificate_details();
                        EditorEventResult::KeepActive
                    }
                }
                _ => EditorEventResult::KeepActive,
            }
        } else {
            EditorEventResult::Inactive
        }
    }

    fn calculate_render_height(&self) -> usize {
        2
    }

    fn get_text_output(&self, text: &mut String) {
        write!(text, "Signing certificate: ").unwrap();
        match self.signature_type {
            SignatureType::None => writeln!(text, "None\n").unwrap(),
            SignatureType::SelfSigned => writeln!(text, "Self-signed\n").unwrap(),
            SignatureType::Certificate => writeln!(text, "{}", self.signed_certificate.as_ref().unwrap().1).unwrap(),
        }
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.highlight.map(|_| 0)
    }

    fn get_empty_highlight_filler(&self) -> (String, String) {
        (String::new(), "<Show details of loaded certificate>".into())
    }

    fn render_modal(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let mut cursor = None;
        if let Some(component) = self.signed_certificate_details.as_mut() {
            cursor = component.render(area, buf);
        }
        if let Some(component) = self.signature_type_question.as_mut() {
            cursor = component.render(area, buf);
        }
        if let Some(component) = self.open_file_dialog.as_mut() {
            cursor = component.render(area, buf, area.height.saturating_sub(4), area.width.saturating_sub(4));
        }
        if let Some(component) = self.error.as_mut() {
            cursor = component.render(area, buf);
        }
        cursor
    }
}
