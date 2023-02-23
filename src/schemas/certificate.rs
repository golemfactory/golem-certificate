use serde::{Deserialize, Serialize};

use self::key_usage::KeyUsage;

use super::{permissions::Permissions, validity_periods::ValidityPeriods};

pub mod key_usage;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    //TODO Add $schema & publicKey & subject
    pub validity_period: ValidityPeriods,
    pub key_usage: KeyUsage,
    pub permissions: Permissions,
}
