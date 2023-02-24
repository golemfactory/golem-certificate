use serde::{Deserialize, Serialize};

use super::{permissions::Permissions, validity_periods::ValidityPeriod};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDescriptor {
    pub node_id: String, //TODO change to ya_client nodeid
    pub validity_period: ValidityPeriod,
    pub permissions: Permissions,
}
