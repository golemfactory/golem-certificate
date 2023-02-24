use serde::{Deserialize, Serialize};

use crate::serde_utils;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignedEnvelope {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub signed_data: serde_json::Value,
    pub signature: Box<Signature>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub signer: Signer,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Signer {
    #[serde(with = "serde_utils::self_signed")]
    SelfSigned,
    Certificate(SignedEnvelope),
    Other(serde_json::Value),
}

#[cfg(test)]
mod should {
    use super::*;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn serialize_self() {
        let signer = Signer::SelfSigned;
        let json = json!("self");

        assert_eq!(serde_json::to_value(&signer).unwrap(), json);
        assert_eq!(serde_json::from_value::<Signer>(json).unwrap(), signer);
    }
}
