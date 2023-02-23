use serde::{Deserialize, Serialize};

use super::{permissions::Permissions, validity_periods::ValidityPeriods};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodePermissions {
    //TODO replace with props from gap
    pub node_id: String,
    pub validity_period: ValidityPeriods,
    pub permissions: Permissions,
}
