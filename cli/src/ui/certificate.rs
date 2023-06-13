use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::{
    self as gcert,
    Signature, SignedCertificate, validate_certificate,
    schemas::{certificate::Certificate, SIGNED_CERTIFICATE_SCHEMA_ID},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tui::{layout::Rect, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::certificate_to_string,
    document_editor::DocumentEditor,
    editors::*,
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth, identity_area,
    },
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CertificateTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    key_usage: Option<gcert::schemas::certificate::key_usage::KeyUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    permissions: Option<gcert::schemas::permissions::Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    public_key: Option<gcert::Key>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    subject: Option<gcert::schemas::subject::Subject>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    validity_period: Option<gcert::schemas::validity_period::ValidityPeriod>,
}

#[derive(Default)]
pub struct CertificateDocumentEditor {
    key_usage_editor: KeyUsageEditor,
    permissions_editor: PermissionsEditor,
    public_key_editor: KeyEditor,
    subject_editor: SubjectEditor,
    validity_period_editor: ValidityPeriodEditor,
}

impl CertificateDocumentEditor {}

impl DocumentEditor for CertificateDocumentEditor {
    fn allow_self_sign(&self) -> bool {
        true
    }

    fn get_document(&self) -> Result<Value> {
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

    fn get_template_json(&self) -> Value {
        let certificate = CertificateTemplate {
            key_usage: Some(self.key_usage_editor.get_key_usage()),
            permissions: Some(self.permissions_editor.get_permissions()),
            public_key: self.public_key_editor.get_key(),
            subject: Some(self.subject_editor.get_subject()),
            validity_period: Some(self.validity_period_editor.get_validity_period()),
        };
        json!({ "certificate": certificate })
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

    fn get_document_type(&self) -> &'static str {
        "Certificate"
    }

    fn create_signed_document(&self, algorithm: gcert::SignatureAlgorithm, signature_value: Vec<u8>, signer: gcert::Signer) -> serde_json::Result<Value> {
        let certificate = self.get_document().unwrap();
        let signed_certificate = SignedCertificate {
            schema: SIGNED_CERTIFICATE_SCHEMA_ID.to_string(),
            certificate,
            signature: Box::new(Signature { algorithm, value: signature_value, signer }),
        };
        serde_json::to_value(signed_certificate)
    }

    fn validate_signed_document(&self, signed_document: Value) -> gcert::Result<Value> {
        validate_certificate(signed_document.clone(), None)
            .map(|_| signed_document)
    }
}
