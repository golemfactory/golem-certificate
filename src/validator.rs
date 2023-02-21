use anyhow::Result;

use crate::schemas::{
    certificate::Certificate,
    node_permissions::NodePermissions,
    signed_envelope::{SignedEnvelope, Signer},
};

//TODO Rafał Proper return value
pub fn validate_envelope(data: &str) -> Result<()> {
    let envelope: SignedEnvelope = serde_json::from_str(data)?;

    let node_permissions: NodePermissions = serde_json::from_str(envelope.signed_data.get())?;

    for signature in &envelope.signatures {
        match &signature.signer {
            Signer::SelfSigned => panic!("NODE JSON CANNOT BE SELF SIGNED"),
            Signer::Certificate(cert_envelope) => {
                let leaf = validate_certificate_chain(&cert_envelope)?;

                //TODO node permission & leaf cert checks here
            }
        }
    }

    Ok(())
}

fn validate_certificate_chain(envelope: &SignedEnvelope) -> Result<Certificate> {
    //TODO Rafał Optimize this algorithm (child is put on stack always)
    let child: Certificate = serde_json::from_str(envelope.signed_data.get())?;

    for signature in &envelope.signatures {
        let parent = match &signature.signer {
            Signer::SelfSigned => serde_json::from_str(envelope.signed_data.get())?,
            Signer::Certificate(parent_envelope) => validate_certificate_chain(&parent_envelope)?,
        };

        //TODO parent & child checks here
    }

    Ok(child)
}
