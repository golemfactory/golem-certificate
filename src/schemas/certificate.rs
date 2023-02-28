use anyhow::Result;
use serde::{Deserialize, Serialize};

use self::key_usage::KeyUsage;

use super::{permissions::Permissions, validity_period::ValidityPeriod};

pub mod key_usage;

pub type Fingerprint = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    pub validity_period: ValidityPeriod,
    pub key_usage: KeyUsage,
    pub permissions: Permissions,
}

impl Certificate {
    pub fn create_cert_id(&self) -> Result<Fingerprint> {
        Ok("mock_fingerprint".into())
    }
}
