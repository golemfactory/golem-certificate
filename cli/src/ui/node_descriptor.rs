use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::{
    self as gcert,
    schemas::{node_descriptor::NodeDescriptor, SIGNED_NODE_DESCRIPTOR_SCHEMA_ID},
    validate_node_descriptor, Signature, SignedNodeDescriptor, Signer,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::node_descriptor_to_string,
    document_editor::DocumentEditor,
    editors::*,
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{default_style, AreaCalculators, CalculateHeight, CalculateWidth},
};

pub struct SignedNodeDescriptorDetails {
    calculate_height: CalculateHeight,
    calculate_width: CalculateWidth,
    render_state: ScrollableTextState,
}

impl SignedNodeDescriptorDetails {
    pub fn new(
        node_descriptor: &SignedNodeDescriptor,
        indent: usize,
        detailed_signer: bool,
        (calculate_height, calculate_width): AreaCalculators,
    ) -> Self {
        let text = node_descriptor_to_string(node_descriptor, indent, detailed_signer);
        Self {
            render_state: ScrollableTextState::new(text),
            calculate_height,
            calculate_width,
        }
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

impl Component for SignedNodeDescriptorDetails {
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

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
        None
    }
}

impl SizedComponent for SignedNodeDescriptorDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        (
            (self.calculate_height)(area.height),
            (self.calculate_width)(area.width),
        )
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NodeDescriptorTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    node_id: Option<ya_client_model::NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    permissions: Option<gcert::schemas::permissions::Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    validity_period: Option<gcert::schemas::validity_period::ValidityPeriod>,
}

#[derive(Default)]
pub struct NodeDescriptorEditor {
    node_id: NodeIdEditor,
    permissions: PermissionsEditor,
    validity_period: ValidityPeriodEditor,
}

impl DocumentEditor for NodeDescriptorEditor {
    fn allow_self_sign(&self) -> bool {
        false
    }

    fn get_document_type(&self) -> &'static str {
        "Node descriptor"
    }

    fn editors_mut(&mut self) -> Vec<&mut dyn EditorComponent> {
        vec![
            &mut self.node_id,
            &mut self.permissions,
            &mut self.validity_period,
        ]
    }

    fn load_template(&mut self, template: Value) {
        if let Some(value) = template.get("nodeDescriptor") {
            if let Ok(template) = serde_json::from_value::<NodeDescriptorTemplate>(value.clone()) {
                self.node_id = NodeIdEditor::new(template.node_id);
                self.permissions = PermissionsEditor::new(template.permissions);
                self.validity_period = ValidityPeriodEditor::new(template.validity_period);
            }
        }
    }

    fn get_document(&self) -> Result<Value> {
        let node_descriptor = NodeDescriptor {
            node_id: self.node_id.get_node_id(),
            permissions: self.permissions.get_permissions(),
            validity_period: self.validity_period.get_validity_period(),
        };
        serde_json::to_value(node_descriptor).map_err(Into::into)
    }

    fn get_template_json(&self) -> Value {
        let template = NodeDescriptorTemplate {
            node_id: Some(self.node_id.get_node_id()),
            permissions: Some(self.permissions.get_permissions()),
            validity_period: Some(self.validity_period.get_validity_period()),
        };
        json!({ "nodeDescriptor": template })
    }

    fn create_signed_document(
        &self,
        algorithm: gcert::SignatureAlgorithm,
        signature_value: Vec<u8>,
        signer: Signer,
    ) -> serde_json::Result<Value> {
        let node_descriptor = self.get_document().unwrap();
        match signer {
            Signer::SelfSigned => unreachable!("Self-signed node descriptors are not allowed"),
            Signer::Certificate(signed_cert) => {
                let signed_node_descriptor = SignedNodeDescriptor {
                    schema: SIGNED_NODE_DESCRIPTOR_SCHEMA_ID.into(),
                    node_descriptor,
                    signature: Signature {
                        algorithm,
                        value: signature_value,
                        signer: signed_cert,
                    },
                };
                serde_json::to_value(signed_node_descriptor)
            }
        }
    }

    fn validate_signed_document(&self, signed_document: Value) -> gcert::Result<Value> {
        validate_node_descriptor(signed_document.clone(), None).map(|_| signed_document)
    }
}
