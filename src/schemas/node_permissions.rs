use serde::{Deserialize, Serialize};

use super::{permissions::Permissions, validity_periods::ValidityPeriod};

//TODO Rafał rename NodeDescriptor
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePermissions {
    pub node_id: String, //TODO change to ya_client nodeid
    pub validity_period: ValidityPeriod,
    pub permissions: Permissions,
}
