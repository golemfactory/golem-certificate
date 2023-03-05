use anyhow::{anyhow, Result};
use chrono::Utc;
use hex::ToHex;
use serde_json::Value;

use crate::{
    cryptography::{create_default_hash, verify_signature_json},
    schemas::{
        certificate::{
            key_usage::validator::{validate_certificates_key_usage, validate_sign_node},
            Certificate, Fingerprint,
        },
        node_descriptor::NodeDescriptor,
        permissions::validator::validate_permissions,
        signature::{SignedCertificate, SignedNodeDescriptor, Signer},
        validity_period::validator::{validate_timestamp, validate_validity_period},
    },
};

use self::validated_data::{ValidatedCertificate, ValidatedNodeDescriptor};

pub mod validated_data;

pub fn validate_certificate_str(data: &str) -> Result<ValidatedCertificate> {
    let value: Value = serde_json::from_str(data)?;
    validate_certificate(value)
}

pub fn validate_certificate(value: Value) -> Result<ValidatedCertificate> {
    validate_schema(
        &value,
        "https://golem.network/schemas/v1/certificate.schema.json",
        "certificate",
    )?;
    let signed_certificate: SignedCertificate = serde_json::from_value(value)?;
    let mut validated_certificate = validate_signed_certificate(&signed_certificate)?;
    validated_certificate
        .certificate_chain_fingerprints
        .reverse();
    Ok(validated_certificate)
}

pub fn validate_node_descriptor_str(data: &str) -> Result<ValidatedNodeDescriptor> {
    let value: Value = serde_json::from_str(data)?;
    validate_node_descriptor(value)
}

pub fn validate_node_descriptor(value: Value) -> Result<ValidatedNodeDescriptor> {
    validate_schema(
        &value,
        "https://golem.network/schemas/v1/node-descriptor.schema.json",
        "node descriptor",
    )?;
    let signed_node_descriptor: SignedNodeDescriptor = serde_json::from_value(value)?;
    let mut validated_node_descriptor = validate_signed_node_descriptor(signed_node_descriptor)?;
    validated_node_descriptor
        .certificate_chain_fingerprints
        .reverse();
    Ok(validated_node_descriptor)
}

fn validate_schema(value: &Value, schema_id: &str, structure_name: &str) -> Result<()> {
    value["$schema"]
        .as_str()
        .map(|schema| {
            if schema == schema_id {
                Ok(())
            } else {
                Err(anyhow!(
                    "Unknown {structure_name} structure with schema: {schema}"
                ))
            }
        })
        .unwrap_or_else(|| Err(anyhow!(
            "Cannot verify {structure_name} structure, schema is not defined"
        )))
}

fn validate_signed_node_descriptor(
    signed_node_descriptor: SignedNodeDescriptor,
) -> Result<ValidatedNodeDescriptor> {
    let node_descriptor: NodeDescriptor =
        serde_json::from_value(signed_node_descriptor.node_descriptor)?;

    let signing_certificate = signed_node_descriptor.signature.signer;
    let validated_certificate = validate_signed_certificate(&signing_certificate)?;

    validate_permissions(
        &validated_certificate.permissions,
        &node_descriptor.permissions,
    )?;
    validate_sign_node(&validated_certificate.key_usage)?;
    validate_validity_period(
        &validated_certificate.validity_period,
        &node_descriptor.validity_period,
    )?;

    validate_timestamp(&node_descriptor.validity_period, Utc::now())?;

    Ok(ValidatedNodeDescriptor {
        certificate_chain_fingerprints: validated_certificate.certificate_chain_fingerprints,
        permissions: node_descriptor.permissions,
        node_id: node_descriptor.node_id,
    })
}

fn create_certificate_fingerprint(signed_certificate: &SignedCertificate) -> Result<Fingerprint> {
    create_fingerprint_for_value(&signed_certificate.certificate)
}

fn create_fingerprint_for_value(value: &Value) -> Result<Fingerprint> {
    create_default_hash(value).map(|binary| binary.encode_hex())
}

fn validate_signed_certificate(
    signed_certificate: &SignedCertificate,
) -> Result<ValidatedCertificate> {
    let parent = match &signed_certificate.signature.signer {
        Signer::SelfSigned => {
            let certificate: Certificate =
                serde_json::from_value(signed_certificate.certificate.clone())?;
            verify_signature_json(
                &signed_certificate.certificate,
                &signed_certificate.signature.value,
                &certificate.public_key,
            )?;
            ValidatedCertificate {
                certificate_chain_fingerprints: vec![],
                permissions: certificate.permissions,
                key_usage: certificate.key_usage,
                validity_period: certificate.validity_period,
            }
        }
        Signer::Certificate(signed_parent) => {
            let parent: Certificate = serde_json::from_value(signed_parent.certificate.clone())?;
            verify_signature_json(
                &signed_certificate.certificate,
                &signed_certificate.signature.value,
                &parent.public_key,
            )?;
            validate_signed_certificate(signed_parent)?
        }
    };

    let certificate: Certificate = serde_json::from_value(signed_certificate.certificate.clone())?;

    validate_permissions(&parent.permissions, &certificate.permissions)?;
    validate_certificates_key_usage(&parent.key_usage, &certificate.key_usage)?;
    validate_validity_period(&parent.validity_period, &certificate.validity_period)?;
    validate_timestamp(&certificate.validity_period, Utc::now())?;

    let mut fingerprints = parent.certificate_chain_fingerprints;
    fingerprints.push(create_certificate_fingerprint(signed_certificate)?);

    Ok(ValidatedCertificate {
        certificate_chain_fingerprints: fingerprints,
        permissions: certificate.permissions,
        key_usage: certificate.key_usage,
        validity_period: certificate.validity_period,
    })
}
