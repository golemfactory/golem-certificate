use serde::{Deserialize, Serialize};

use crate::cryptography::Key;

use self::key_usage::KeyUsage;

use super::{permissions::Permissions, subject::Subject, validity_period::ValidityPeriod};

pub mod key_usage;

pub type Fingerprint = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub key_usage: KeyUsage,
    pub permissions: Permissions,
    pub public_key: Key,
    pub subject: Subject,
    pub validity_period: ValidityPeriod,
}
