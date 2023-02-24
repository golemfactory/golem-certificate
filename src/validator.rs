use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::schemas::{
    certificate::{
        key_usage::validator::{validate_certificates_key_usage, validate_sign_node},
        Certificate,
    },
    node_descriptor::NodeDescriptor,
    permissions::validator::validate_permissions,
    signed_envelope::{SignedEnvelope, Signer},
    validity_periods::validator::{validate_timestamp, validate_validity_periods},
};

use self::{error::ValidationError, success::Success, certificate_descriptor::CertificateId};

pub mod certificate_descriptor;
pub mod error;
pub mod success;

//TODO Rafał proper return value
pub fn validate(data: &str) -> Result<Success> {
    let value: serde_json::Value = serde_json::from_str(data)?;
    let schema = value["$schema"]
        .as_str()
        .ok_or(anyhow!("no schema provided"))?;

    match schema {
        "https://golem.network/schemas/v1/signed-envelope.schema.json" => {
            let signed_data_schema = value["signedData"]["$schema"]
                .as_str()
                .ok_or(anyhow!("no schema provided"))?;

            match signed_data_schema {
                "https://golem.network/schemas/v1/node.schema.json" => {
                    let envelope: SignedEnvelope = serde_json::from_value(value)?;
                    validate_node_descriptor_envelope(envelope)
                
                },

                "https://golem.network/schemas/v1/certificate.schema.json" => {
                    let envelope: SignedEnvelope = serde_json::from_value(value)?;
                    validate_certificate_envelope(envelope)
                }
                signed_data_schema => Err(anyhow!(
                    "Following schema in signed data in envelope is not supported yet: {signed_data_schema}"
                )),
            }
        }
        schema => Err(anyhow!(
            "Following schema is not supported yet for validation: {schema}"
        )),
    }
}

fn validate_node_descriptor_envelope(envelope: SignedEnvelope) -> Result<Success> {
    let node_descriptor: NodeDescriptor = serde_json::from_value(envelope.signed_data)?;

    let mut certs = vec![];

    match &envelope.signature.signer {
        Signer::Other(_) => return Err(anyhow!("Other form of signer is not supported yet")),
        Signer::SelfSigned => return Err(anyhow!("Node Permissions cannot be self-signed")),
        Signer::Certificate(cert_envelope) => {
            let leaf = validate_certificate(&cert_envelope, &mut certs)?;

            validate_permissions(&leaf.permissions, &node_descriptor.permissions)?;
            validate_sign_node(&leaf.key_usage)?;
            validate_validity_periods(
                &leaf.validity_period,
                &node_descriptor.validity_period,
            )?;
        }
    }

    validate_timestamp(&node_descriptor.validity_period, Utc::now())?;

    Ok(Success::NodeDescriptor { node_id: node_descriptor.node_id, permissions: node_descriptor.permissions, certs })
}

fn validate_certificate_envelope(envelope: SignedEnvelope) -> Result<Success> {
    let mut certs = vec![];

    let leaf = validate_certificate(&envelope, &mut certs)?;

    validate_timestamp(&leaf.validity_period, Utc::now())?;

    Ok(Success::Certificate { permissions: leaf.permissions, certs })
}

fn validate_certificate(envelope: &SignedEnvelope, validated_certs: &mut Vec<CertificateId>) -> Result<Certificate> {
    //TODO Rafał Optimize this algorithm (child is put on stack always)
    let child: Certificate = serde_json::from_value(envelope.signed_data.clone())?;

    let parent = match &envelope.signature.signer {
        Signer::Other(_) => return Err(anyhow!("Other form of signer is not supported yet")),
        Signer::SelfSigned => serde_json::from_value(envelope.signed_data.clone())?,
        Signer::Certificate(parent_envelope) => validate_certificate(&parent_envelope, validated_certs)?,
    };

    validate_permissions(&parent.permissions, &child.permissions)?;
    validate_certificates_key_usage(&parent.key_usage, &child.key_usage)?;
    validate_validity_periods(&parent.validity_period, &child.validity_period)?;

    let cert_id = child.create_cert_id()?;
    validated_certs.push(cert_id);

    Ok(child)
}
