use serde::{Deserialize, Serialize};

use crate::cryptography::{EncryptionAlgorithm, HashAlgorithm};
use crate::serde_utils::{bytes_to_hex, hex_to_bytes};

pub const SIGNED_NODE_DESCRIPTOR_SCHEMA_ID: &str =
    "https://schemas.golem.network/v1/node-descriptor.schema.json";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignedNodeDescriptor {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub node_descriptor: serde_json::Value,
    pub signature: Signature<SignedCertificate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Signature<T> {
    pub algorithm: SignatureAlgorithm,
    #[serde(serialize_with = "bytes_to_hex", deserialize_with = "hex_to_bytes")]
    pub value: Vec<u8>,
    pub signer: T,
}

impl Signature<Signer> {
    pub fn create_self_signed(algorithm: SignatureAlgorithm, value: Vec<u8>) -> Self {
        Signature::<Signer> {
            algorithm,
            value,
            signer: Signer::SelfSigned,
        }
    }
}

impl Signature<SignedCertificate> {
    pub fn create(
        algorithm: SignatureAlgorithm,
        value: Vec<u8>,
        certificate: SignedCertificate,
    ) -> Self {
        Signature::<SignedCertificate> {
            algorithm,
            value,
            signer: certificate,
        }
    }
}

pub const SIGNED_CERTIFICATE_SCHEMA_ID: &str =
    "https://schemas.golem.network/v1/certificate.schema.json";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignedCertificate {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub certificate: serde_json::Value,
    pub signature: Box<Signature<Signer>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SignatureAlgorithm {
    pub hash: HashAlgorithm,
    pub encryption: EncryptionAlgorithm,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Signer {
    #[serde(with = "crate::serde_utils::self_signed")]
    SelfSigned,
    Certificate(SignedCertificate),
}

#[cfg(test)]
mod should {
    use super::*;

    use pretty_assertions::{assert_eq, assert_matches};
    use serde_json::json;

    #[test]
    fn serialize_self() {
        let signer = Signer::SelfSigned;
        let json = json!("self");

        assert_eq!(serde_json::to_value(&signer).unwrap(), json);
        assert_matches!(
            serde_json::from_value::<Signer>(json).unwrap(),
            Signer::SelfSigned
        );
    }
}
