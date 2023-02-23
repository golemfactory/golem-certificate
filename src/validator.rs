use anyhow::{anyhow, Result};
use chrono::Utc;

use crate::schemas::{
    certificate::{
        key_usage::validator::{validate_certificates_key_usage, validate_sign_node},
        Certificate,
    },
    node_permissions::NodePermissions,
    permissions::validator::validate_permissions,
    signed_envelope::{SignedEnvelope, Signer},
    validity_periods::validator::{validate_timestamp, validate_validity_periods},
};

//TODO Rafał proper return value
pub fn validate(data: &str) -> Result<()> {
    let value: serde_json::Value = serde_json::from_str(data)?;
    let schema = value["$schema"]
        .as_str()
        .ok_or(anyhow!("no schema provided"))?;

    match schema {
        "https://golem.network/schemas/v0/signed-envelop.schema.json" => {
            let signed_data_schema = value["signedData"]["$schema"]
                .as_str()
                .ok_or(anyhow!("no schema provided"))?;

            match signed_data_schema {
                "todo-node-permissions-schema" => {
                    let envelope: SignedEnvelope = serde_json::from_value(value)?;
                    validate_node_permissions_envelope(envelope)},

                "https://golem.network/schemas/v1/certificate.schema.json" => {
                    let envelope: SignedEnvelope = serde_json::from_value(value)?;
                    validate_certificate(&envelope)?;
                    Ok(())
                }
                signed_data_schema => Err(anyhow!(
                    "Following schema in signed data in envelope is not supported yet: {signed_data_schema}"
                )),
            }
        }
        "https://golem.network/schemas/v1/certificate.schema.json" => {
            //TODO Do we want to validate certs without envelope?
            Ok(())
        }
        schema => Err(anyhow!("Following schema is not supported yet: {schema}")),
    }
}

fn validate_node_permissions_envelope(envelope: SignedEnvelope) -> Result<()> {
    let node_permissions: NodePermissions = serde_json::from_value(envelope.signed_data)?;

    for signature in &envelope.signatures {
        match &signature.signer {
            Signer::SelfSigned => return Err(anyhow!("Node Permissions cannot be self-signed")),
            Signer::Other(_) => return Err(anyhow!("Other form of signer is not supported yet")),
            Signer::Certificate(cert_envelope) => {
                let leaf = validate_certificate(&cert_envelope)?;

                //TODO Rafał Do we want to use UTC now here?
                validate_timestamp(&node_permissions.validity_period, Utc::now())?;
                validate_permissions(&leaf.permissions, &node_permissions.permissions)?;
                validate_sign_node(&leaf.key_usage)?;
                validate_validity_periods(
                    &leaf.validity_period,
                    &node_permissions.validity_period,
                )?;
            }
        }
    }

    Ok(())
}

fn validate_certificate(envelope: &SignedEnvelope) -> Result<Certificate> {
    //TODO Rafał Optimize this algorithm (child is put on stack always)
    let child: Certificate = serde_json::from_value(envelope.signed_data.clone())?;

    for signature in &envelope.signatures {
        let parent = match &signature.signer {
            Signer::Other(_) => return Err(anyhow!("Other form of signer is not supported yet")),
            Signer::SelfSigned => serde_json::from_value(envelope.signed_data.clone())?,
            Signer::Certificate(parent_envelope) => validate_certificate(&parent_envelope)?,
        };

        validate_permissions(&parent.permissions, &child.permissions)?;
        validate_certificates_key_usage(&parent.key_usage, &child.key_usage)?;
        validate_validity_periods(&parent.validity_period, &child.validity_period)?;
    }

    Ok(child)
}
