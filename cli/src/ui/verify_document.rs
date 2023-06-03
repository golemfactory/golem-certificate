use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::Result;
use golem_certificate::{validate_certificate, validate_node_descriptor, Error::*, SignedCertificate, SignedNodeDescriptor};
use serde_json::Value;
use tui::widgets::{Block, BorderType, Widget, Padding, Borders};

use super::certificate::SignedCertificateDetails;
use super::modal::{ModalMessage, ModalWithComponent};
use super::open_file_dialog::OpenFileDialog;
use super::util::{Component, ComponentStatus, default_style};

enum VerifiedDocument {
    Certificate(SignedCertificate),
    NodeDescriptor(SignedNodeDescriptor),
}

pub struct VerifyDocument {
    open_file_dialog: OpenFileDialog,
    modal: Option<Box<dyn Component>>,
}

impl VerifyDocument {
    pub fn new() -> Result<Self> {
        let open_file_dialog = OpenFileDialog::new()?;
        let verify_document = Self {
            open_file_dialog,
            modal: None,
        };
        Ok(verify_document)
    }
}

impl Component for VerifyDocument {
    fn render(&mut self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let block = Block::default()
            .title("Verify document")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(default_style())
            .padding(Padding::vertical(1));
        let inner_area = block.inner(area);
        block.render(area, buf);

        self.open_file_dialog.render(inner_area, buf);
        if let Some(component) = &mut self.modal {
            component.render(inner_area, buf);
        }
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> Result<ComponentStatus> {
        if let Some(component) = &mut self.modal {
            if component.handle_key_event(key_event)? != ComponentStatus::Active {
                self.modal = None;
            }
            Ok(ComponentStatus::Active)
        } else {
            match self.open_file_dialog.handle_key_event(key_event)? {
                ComponentStatus::Closed => {
                    if let Some(path) = self.open_file_dialog.selected.as_mut() {
                        let modal: Box<dyn Component> = match verify_selected_file(&path) {
                            Ok(VerifiedDocument::Certificate(cert)) => show_cert_details(path, &cert),
                            Ok(VerifiedDocument::NodeDescriptor(node_descriptor)) => show_node_descriptor_details(path, node_descriptor),
                            Err(err) => show_error(path, err),
                        };
                        self.modal = Some(modal);
                    }
                    Ok(ComponentStatus::Active)
                },
                s => Ok(s),
            }
        }
    }
}

fn verify_selected_file(path: &PathBuf) -> Result<VerifiedDocument, String> {
    read_to_string(&path)
        .map_err(|err| format!("Cannot read file ({})", err))
        .and_then(|contents| {
            serde_json::from_str::<Value>(&contents)
                .map_err(|_| "File contents is not JSON".into())
        })
        .and_then(|json| verify_json(json))
}

fn verify_json(json: Value) -> Result<VerifiedDocument, String> {
    match validate_certificate(json.clone(), None) {
        Ok(_) => {
            let signed_cert = serde_json::from_value(json.clone()).unwrap();
            Ok(VerifiedDocument::Certificate(signed_cert))
        }
        Err(UnsupportedSchema { .. }) => {
            validate_node_descriptor(json.clone())
                .map(|_| {
                    let signed_node_descriptor = serde_json::from_value(json).unwrap();
                    VerifiedDocument::NodeDescriptor(signed_node_descriptor)
                })
        }
        Err(e) => Err(e)
    }.map_err(|err| err.to_string())
}

fn show_cert_details(path: &PathBuf, cert: &SignedCertificate) -> Box<dyn Component> {
    let component = SignedCertificateDetails::new(cert, 2, true);
    let modal = ModalWithComponent::new(path.to_string_lossy(), Box::new(component));
    Box::new(modal)
}

fn show_node_descriptor_details(path: &PathBuf, node_descriptor: SignedNodeDescriptor) -> Box<dyn Component> {
    let title = "Node Descriptor";
    let message = "All looks good!";
    let modal = ModalMessage::new(title, message);
    Box::new(modal)
}

fn show_error(path: &PathBuf, err: String) -> Box<dyn Component> {
    let title = "Error during verification";
    let message = format!("Verifying {}\nError: {}", path.to_string_lossy(), err);
    let modal = ModalMessage::new(title, message);
    Box::new(modal)
}
