use serde::{Deserialize, Serialize};
use ya_client_model::NodeId;

use super::{permissions::Permissions, validity_period::ValidityPeriod};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeDescriptor {
    pub node_id: NodeId,
    pub validity_period: ValidityPeriod,
    pub permissions: Permissions,
}
