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
    Error,
};

use self::validated_data::{ValidatedCertificate, ValidatedNodeDescriptor};

pub mod validated_data;

pub fn validate_certificate_str(data: &str) -> Result<ValidatedCertificate, Error> {
    let value: Value =
        serde_json::from_str(data).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    validate_certificate(value)
}

pub fn validate_certificate(value: Value) -> Result<ValidatedCertificate, Error> {
    validate_schema(
        &value,
        "https://golem.network/schemas/v1/certificate.schema.json",
        "certificate",
    )?;
    let signed_certificate: SignedCertificate =
        serde_json::from_value(value).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    let mut validated_certificate = validate_signed_certificate(&signed_certificate)?;
    validated_certificate
        .certificate_chain_fingerprints
        .reverse();
    Ok(validated_certificate)
}

pub fn validate_node_descriptor_str(data: &str) -> Result<ValidatedNodeDescriptor, Error> {
    let value: Value =
        serde_json::from_str(data).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    validate_node_descriptor(value)
}

pub fn validate_node_descriptor(value: Value) -> Result<ValidatedNodeDescriptor, Error> {
    validate_schema(
        &value,
        "https://golem.network/schemas/v1/node-descriptor.schema.json",
        "node descriptor",
    )?;
    let signed_node_descriptor: SignedNodeDescriptor =
        serde_json::from_value(value).map_err(|e| Error::InvalidFormat(e.to_string()))?;
    let mut validated_node_descriptor = validate_signed_node_descriptor(signed_node_descriptor)?;
    validated_node_descriptor
        .certificate_chain_fingerprints
        .reverse();
    Ok(validated_node_descriptor)
}

fn validate_schema(value: &Value, schema_id: &str, structure_name: &str) -> Result<(), Error> {
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
            Err(Error::UnsupportedSchema {
                schema: "".to_owned(),
                structure_name: structure_name.to_owned(),
            })
        })
}

fn validate_signed_node_descriptor(
    signed_node_descriptor: SignedNodeDescriptor,
) -> Result<ValidatedNodeDescriptor, Error> {
    let node_descriptor: NodeDescriptor =
        serde_json::from_value(signed_node_descriptor.node_descriptor.clone())
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

    let signing_certificate = signed_node_descriptor.signature.signer;
    let validated_certificate = validate_signed_certificate(&signing_certificate)?;

    let leaf_certificate: Certificate = serde_json::from_value(signing_certificate.certificate)
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;
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

    validate_timestamp(&node_descriptor.validity_period, Utc::now())?;

    Ok(ValidatedNodeDescriptor {
        certificate_chain_fingerprints: validated_certificate.certificate_chain_fingerprints,
        permissions: node_descriptor.permissions,
        node_id: node_descriptor.node_id,
    })
}

fn create_certificate_fingerprint(
    signed_certificate: &SignedCertificate,
) -> Result<Fingerprint, Error> {
    create_fingerprint_for_value(&signed_certificate.certificate)
}

fn create_fingerprint_for_value(value: &Value) -> Result<Fingerprint, Error> {
    create_default_hash(value).map(|binary| binary.encode_hex())
}

fn validate_signed_certificate(
    signed_certificate: &SignedCertificate,
) -> Result<ValidatedCertificate, Error> {
    let parent = match &signed_certificate.signature.signer {
        Signer::SelfSigned => {
            let certificate: Certificate =
                serde_json::from_value(signed_certificate.certificate.clone())
                    .map_err(|e| Error::InvalidFormat(e.to_string()))?;
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
                .map_err(|e| Error::InvalidFormat(e.to_string()))?;
            verify_signature_json(
                &signed_certificate.certificate,
                &signed_certificate.signature.value,
                &parent.public_key,
            )?;
            validate_signed_certificate(signed_parent)?
        }
    };

    let certificate: Certificate = serde_json::from_value(signed_certificate.certificate.clone())
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;

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
        subject: certificate.subject,
    })
}
