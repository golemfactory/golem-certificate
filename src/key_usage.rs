use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::serde_utils;

pub mod validator;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum KeyUsage {
    #[serde(with = "serde_utils::all")]
    All,
    Usages(HashSet<Usage>),
}

// #[derive(Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Usages {
//     pub usages: HashSet<Usage>,
// }

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Usage {
    SignCertificate,
    SignManifest,
    SignNode,
}

#[cfg(test)]
mod should {
    use super::*;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn serialize_all() {
        let key_usage = KeyUsage::All;

        assert_eq!(serde_json::to_value(&key_usage).unwrap(), json!("all"));
    }

    #[test]
    fn serialize_outbound_unrestricted() {
        let key_usage = KeyUsage::Usages([Usage::SignCertificate].into());

        assert_eq!(
            serde_json::to_value(&key_usage).unwrap(),
            json!(["signCertificate"])
        );
    }
}
