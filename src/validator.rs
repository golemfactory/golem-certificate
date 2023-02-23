use anyhow::{anyhow, Result};

use crate::schemas::{
    certificate::Certificate,
    node_permissions::NodePermissions,
    signed_envelope::{SignedEnvelope, Signer},
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
            Signer::Certificate(cert_envelope) => {
                let leaf = validate_certificate(&cert_envelope)?;

                //TODO node permission & leaf cert checks here
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
            Signer::SelfSigned => serde_json::from_value(envelope.signed_data.clone())?,
            Signer::Certificate(parent_envelope) => validate_certificate(&parent_envelope)?,
        };

        //TODO parent & child checks here
    }

    Ok(child)
}
