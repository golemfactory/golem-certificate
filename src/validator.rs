use chrono::{DateTime, Utc};
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
        SIGNED_CERTIFICATE_SCHEMA_ID, SIGNED_NODE_DESCRIPTOR_SCHEMA_ID,
    },
    Error, Result,
};

use self::validated_data::{ValidatedCertificate, ValidatedNodeDescriptor};

pub mod validated_data;

/// Deserializes and validates certificate.
/// # Arguments
/// * `data` serialized certificate
/// * `timestamp` optional timestamp to verify validity
pub fn validate_certificate_str(
    data: &str,
    timestamp: Option<DateTime<Utc>>,
) -> Result<ValidatedCertificate> {
    let value: Value = serde_json::from_str(data).map_err(|e| Error::InvalidJson(e.to_string()))?;
    validate_certificate(value, timestamp)
}

/// Validates certificate.
/// # Arguments
/// * `value` certificate
/// * `timestamp` optional timestamp to verify validity
pub fn validate_certificate(
    value: Value,
    timestamp: Option<DateTime<Utc>>,
) -> Result<ValidatedCertificate> {
    validate_schema(&value, SIGNED_CERTIFICATE_SCHEMA_ID, "certificate")?;
    let signed_certificate: SignedCertificate = serde_json::from_value(value)
        .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;
    let mut validated_certificate = validate_signed_certificate(&signed_certificate, timestamp)?;
    validated_certificate
        .certificate_chain_fingerprints
        .reverse();
    Ok(validated_certificate)
}

/// Deserializes and validates node descriptor.
/// # Arguments
/// * `data` serialized node descriptor
/// * `timestamp` optional timestamp to verify validity
pub fn validate_node_descriptor_str(
    data: &str,
    timestamp: Option<DateTime<Utc>>,
) -> Result<ValidatedNodeDescriptor> {
    let value: Value = serde_json::from_str(data).map_err(|e| Error::InvalidJson(e.to_string()))?;
    validate_node_descriptor(value, timestamp)
}

/// Validates node descriptor.
/// # Arguments
/// * `value` node descriptor
/// * `timestamp` optional timestamp to verify validity
pub fn validate_node_descriptor(
    value: Value,
    timestamp: Option<DateTime<Utc>>,
) -> Result<ValidatedNodeDescriptor> {
    validate_schema(&value, SIGNED_NODE_DESCRIPTOR_SCHEMA_ID, "node descriptor")?;
    let signed_node_descriptor: SignedNodeDescriptor = serde_json::from_value(value)
        .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;
    let mut validated_node_descriptor =
        validate_signed_node_descriptor(signed_node_descriptor, timestamp)?;
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
                Err(Error::UnsupportedSchema {
                    schema: schema.to_owned(),
                    structure_name: structure_name.to_owned(),
                })
            }
        })
        .unwrap_or_else(|| {
            Err(Error::JsonDoesNotConformToSchema(format!(
                "Missing `schema` property in {structure_name}"
            )))
        })
}

/// Validates signed node descriptor.
/// # Arguments
/// * `signed_node_descriptor`
/// * `timestamp` optional timestamp to verify validity of the leaf certificate (last certificate in the chain).
///    Validity periods of parent (issuer) certificates from the chain must fully include validity period of a child.
fn validate_signed_node_descriptor(
    signed_node_descriptor: SignedNodeDescriptor,
    timestamp: Option<DateTime<Utc>>,
) -> Result<ValidatedNodeDescriptor> {
    let node_descriptor: NodeDescriptor =
        serde_json::from_value(signed_node_descriptor.node_descriptor.clone())
            .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;

    let signing_certificate = signed_node_descriptor.signature.signer;
    let validated_certificate = validate_signed_certificate(&signing_certificate, None)?;

    let leaf_certificate: Certificate = serde_json::from_value(signing_certificate.certificate)
        .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;
    verify_signature_json(
        &signed_node_descriptor.node_descriptor,
        &signed_node_descriptor.signature.value,
        &leaf_certificate.public_key,
    )?;

    validate_permissions(
        &validated_certificate.permissions,
        &node_descriptor.permissions,
    )?;
    validate_sign_node(&validated_certificate.key_usage)?;
    validate_validity_period(
        &validated_certificate.validity_period,
        &node_descriptor.validity_period,
    )?;

    timestamp
        .map(|ts| validate_timestamp(&node_descriptor.validity_period, ts))
        .unwrap_or(Ok(()))?;

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

/// Validates signed certificate.
/// # Arguments
/// * `signed_certificate`
/// * `timestamp` optional timestamp to verify validity of the leaf certificate (last certificate in the chain).
///    Validity periods of parent (issuer) certificates from the chain must fully include validity period of a child.
fn validate_signed_certificate(
    signed_certificate: &SignedCertificate,
    timestamp: Option<DateTime<Utc>>,
) -> Result<ValidatedCertificate> {
    let parent = match &signed_certificate.signature.signer {
        Signer::SelfSigned => {
            let certificate: Certificate =
                serde_json::from_value(signed_certificate.certificate.clone())
                    .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;
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
                subject: certificate.subject,
            }
        }
        Signer::Certificate(signed_parent) => {
            let parent: Certificate = serde_json::from_value(signed_parent.certificate.clone())
                .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;
            verify_signature_json(
                &signed_certificate.certificate,
                &signed_certificate.signature.value,
                &parent.public_key,
            )?;
            validate_signed_certificate(signed_parent, None)?
        }
    };

    let certificate: Certificate = serde_json::from_value(signed_certificate.certificate.clone())
        .map_err(|e| Error::JsonDoesNotConformToSchema(e.to_string()))?;

    validate_permissions(&parent.permissions, &certificate.permissions)?;
    validate_certificates_key_usage(&parent.key_usage, &certificate.key_usage)?;
    validate_validity_period(&parent.validity_period, &certificate.validity_period)?;
    timestamp
        .map(|ts| validate_timestamp(&certificate.validity_period, ts))
        .unwrap_or(Ok(()))?;

    let mut fingerprints = parent.certificate_chain_fingerprints;
    fingerprints.push(create_certificate_fingerprint(signed_certificate)?);

    Ok(ValidatedCertificate {
        certificate_chain_fingerprints: fingerprints,
        permissions: certificate.permissions,
        key_usage: certificate.key_usage,
        validity_period: certificate.validity_period,
        subject: certificate.subject,
    })
}
