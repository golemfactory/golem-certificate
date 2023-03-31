use chrono::{DateTime, Utc};

use crate::schemas::validity_period::ValidityPeriod;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expired, it was valid to {0}")]
    Expired(DateTime<Utc>),
    #[error("Not valid yet: it will be valid from {0}")]
    NotValidYet(DateTime<Utc>),
    #[error("Child cannot extend time periods, parent: {parent:?}, child: {child:?}")]
    ValidityPeriodExtended {
        parent: ValidityPeriod,
        child: ValidityPeriod,
    },
    #[error("Permissions extended: {0}")]
    PermissionsExtended(String),
    #[error("Key usage extended: {0}")]
    KeyUsageExtended(String),
    #[error("Not permitted: ")]
    NotPermitted(String),
    #[error("TODO")]
    InvalidSignature(String),
    #[error("TODO")]
    InvalidSchema(String),
}
