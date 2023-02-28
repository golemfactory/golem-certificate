use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::schemas::{
    certificate::{
        key_usage::validator::{validate_certificates_key_usage, validate_sign_node},
        Certificate, Fingerprint,
    },
    node_descriptor::NodeDescriptor,
    permissions::validator::validate_permissions,
    signed_envelope::{SignedEnvelope, Signer},
    validity_period::validator::{validate_timestamp, validate_validity_period},
};

use self::validated_data::{ValidatedCert, ValidatedData, ValidatedNodeDescriptor};

pub mod validated_data;

pub fn validate_golem_certificate(data: &str) -> Result<ValidatedCert> {
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
pub fn validate_node_descriptor(data: &str) -> Result<ValidatedNodeDescriptor> {
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

fn validate_node_descriptor_envelope(envelope: SignedEnvelope) -> Result<ValidatedNodeDescriptor> {
    let node_descriptor: NodeDescriptor = serde_json::from_value(envelope.signed_data)?;

    let mut certs = vec![];

    match &envelope.signature.signer {
        Signer::Other(_) => return Err(anyhow!("Other form of signer is not supported yet")),
        Signer::SelfSigned => return Err(anyhow!("Node Permissions cannot be self-signed")),
        Signer::Certificate(cert_envelope) => {
            let leaf = validate_certificate(cert_envelope, &mut certs)?;

            validate_permissions(&leaf.permissions, &node_descriptor.permissions)?;
            validate_sign_node(&leaf.key_usage)?;
            validate_validity_period(&leaf.validity_period, &node_descriptor.validity_period)?;
        }
    }

    validate_timestamp(&node_descriptor.validity_period, Utc::now())?;

    Ok(ValidatedNodeDescriptor {
        descriptor: node_descriptor,
        chain: certs,
    })
}

fn validate_certificate_envelope(envelope: SignedEnvelope) -> Result<ValidatedCert> {
    let mut certs = vec![];

    let leaf = validate_certificate(&envelope, &mut certs)?;

    validate_timestamp(&leaf.validity_period, Utc::now())?;

    Ok(ValidatedCert {
        cert: leaf,
        chain: certs,
    })
}

fn validate_certificate(
    envelope: &SignedEnvelope,
    validated_certs: &mut Vec<Fingerprint>,
) -> Result<Certificate> {
    let parent = match &envelope.signature.signer {
        Signer::Other(_) => return Err(anyhow!("Other form of signer is not supported yet")),
        Signer::SelfSigned => serde_json::from_value(envelope.signed_data.clone())?,
        Signer::Certificate(parent_envelope) => {
            validate_certificate(parent_envelope, validated_certs)?
        }
    };

    let child: Certificate = serde_json::from_value(envelope.signed_data.clone())?;

    validate_permissions(&parent.permissions, &child.permissions)?;
    validate_certificates_key_usage(&parent.key_usage, &child.key_usage)?;
    validate_validity_period(&parent.validity_period, &child.validity_period)?;

    let cert_id = child.create_cert_id()?;
    validated_certs.push(cert_id);

    Ok(child)
}
