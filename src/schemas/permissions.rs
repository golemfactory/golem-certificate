use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::serde_utils;

pub mod validator;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Permissions {
    #[serde(with = "serde_utils::all")]
    All,
    Object(PermissionDetails),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDetails {
    pub outbound: Option<OutboundPermissions>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum OutboundPermissions {
    Unrestricted,
    Urls(HashSet<Url>),
}

#[cfg(test)]
mod should {
    use super::*;

    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn serialize_and_deserialize_all() {
        let permissions = Permissions::All;
        let json = json!("all");

        assert_eq!(serde_json::to_value(&permissions).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<Permissions>(json).unwrap(),
            permissions
        );
    }

    #[test]
    fn serialize_and_deserialize_outbound_unrestricted() {
        let permissions = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });
        let json = json!({
            "outbound": "unrestricted"
        });

        assert_eq!(serde_json::to_value(&permissions).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<Permissions>(json).unwrap(),
            permissions
        );
    }

    #[test]
    fn serialize_outbound_urls() {
        let permissions = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Urls(
                [Url::parse("https://example.net/").unwrap()].into(),
            )),
        });
        let json = json!({
            "outbound": {
                "urls": ["https://example.net/"]
            }
        });

        assert_eq!(serde_json::to_value(&permissions).unwrap(), json);
        assert_eq!(
            serde_json::from_value::<Permissions>(json).unwrap(),
            permissions
        );
    }
}
