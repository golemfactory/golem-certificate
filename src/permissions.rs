use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use url::Url;

mod serde_utils;

pub mod validator;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Permissions {
    #[serde(with = "serde_utils::all")]
    All,
    Object(PermissionDetails),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PermissionDetails {
    pub outbound: Option<OutboundPermissions>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    fn serialize_all() {
        let permissions = Permissions::All;

        assert_eq!(serde_json::to_value(&permissions).unwrap(), json!("all"));
    }

    #[test]
    fn serialize_outbound_unrestricted() {
        let permissions = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });

        assert_eq!(
            serde_json::to_value(&permissions).unwrap(),
            json!({
                "outbound": "unrestricted"
            })
        );
    }

    #[test]
    fn serialize_outbound_urls() {
        let permissions = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Urls(
                [Url::parse("https://example.net/").unwrap()].into(),
            )),
        });

        assert_eq!(
            serde_json::to_value(&permissions).unwrap(),
            json!({
                "outbound": {
                    "urls": ["https://example.net/"]
                }
            })
        );
    }
}
