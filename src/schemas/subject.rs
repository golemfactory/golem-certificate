use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Subject {
    /// "The subject's name that is displayed when processing this certificate"
    pub display_name: String,
    /// "Contact information of the subject"
    pub contact: Contact,
    /// additional properties included in the certificate
    #[serde(flatten)]
    pub additional_properties: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    /// "Contact email"
    pub email: String,
    #[serde(flatten)]
    pub additional_properties: HashMap<String, Value>,
}
