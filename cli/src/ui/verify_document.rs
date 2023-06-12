use std::{fs, path::PathBuf};

use anyhow::Result;
use crossterm::event::KeyEvent;
use golem_certificate::{
    validate_certificate, validate_node_descriptor, Error::*, SignedCertificate,
    SignedNodeDescriptor,
};
use serde_json::Value;
use tui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, BorderType, Borders, Padding, Widget},
};

use super::{
    component::*,
    certificate::SignedCertificateDetails,
    modal::{ModalMessage, ModalWithSizedComponent},
    node_descriptor::SignedNodeDescriptorDetails,
    open_file_dialog::OpenFileDialog,
    util::{default_style, CalculateHeight, CalculateWidth},
};

pub enum VerifiedDocument {
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
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
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
                            Ok(VerifiedDocument::Certificate(cert)) => {
                                show_cert_details(path, &cert)
                            }
                            Ok(VerifiedDocument::NodeDescriptor(node_descriptor)) => {
                                show_node_descriptor_details(path, &node_descriptor)
                            }
                            Err(err) => show_error(path, err),
                        };
                        self.modal = Some(modal);
                    }
                    Ok(ComponentStatus::Active)
                }
                s => Ok(s),
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let block = Block::default()
            .title("Verify document")
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(default_style())
            .padding(Padding::vertical(1));
        let inner_area = block.inner(area);
        block.render(area, buf);

        let browser_cursor = self.open_file_dialog.render(inner_area, buf);
        if let Some(component) = &mut self.modal {
            component.render(inner_area, buf)
        } else {
            browser_cursor
        }
    }
}

fn verify_selected_file(path: &PathBuf) -> Result<VerifiedDocument, String> {
    fs::read_to_string(&path)
        .map_err(|err| format!("Cannot read file ({})", err))
        .and_then(|contents| {
            serde_json::from_str::<Value>(&contents).map_err(|_| "File contents is not JSON".into())
        })
        .and_then(|json| verify_json(json))
}

fn verify_json(json: Value) -> Result<VerifiedDocument, String> {
    match validate_certificate(json.clone(), None) {
        Ok(_) => {
            let signed_cert = serde_json::from_value(json.clone()).unwrap();
            Ok(VerifiedDocument::Certificate(signed_cert))
        }
        Err(UnsupportedSchema { .. }) => validate_node_descriptor(json.clone()).map(|_| {
            let signed_node_descriptor = serde_json::from_value(json).unwrap();
            VerifiedDocument::NodeDescriptor(signed_node_descriptor)
        }),
        Err(e) => Err(e),
    }
    .map_err(|err| err.to_string())
}

fn show_cert_details(path: &PathBuf, cert: &SignedCertificate) -> Box<dyn Component> {
    let component = SignedCertificateDetails::new(cert, 2, true, get_area_calculators());
    let modal = ModalWithSizedComponent::new(path.to_string_lossy(), Box::new(component));
    Box::new(modal)
}

fn show_node_descriptor_details(
    path: &PathBuf,
    node_descriptor: &SignedNodeDescriptor,
) -> Box<dyn Component> {
    let component =
        SignedNodeDescriptorDetails::new(node_descriptor, 2, true, get_area_calculators());
    let modal = ModalWithSizedComponent::new(path.to_string_lossy(), Box::new(component));
    Box::new(modal)
}

fn show_error(path: &PathBuf, err: String) -> Box<dyn Component> {
    let title = "Error during verification";
    let message = format!("Verifying {}\nError: {}", path.to_string_lossy(), err);
    let modal = ModalMessage::new(title, message);
    Box::new(modal)
}

fn get_area_calculators() -> (CalculateHeight, CalculateWidth) {
    let calculate_height = |height: u16| (height * 9) / 10;
    let calculate_width = |width: u16| (width * 8) / 10;
    (Box::new(calculate_height), Box::new(calculate_width))
}
