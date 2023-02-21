use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePermissions {
    //TODO replace with props from gap
    pub node_id: String,
}
