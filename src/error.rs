use chrono::{DateTime, Utc};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Expired: was valid to {0}")]
    Expired(DateTime<Utc>),
    #[error("Not valid yet: will be valid from {0}")]
    NotValidYet(DateTime<Utc>),
    #[error("Validity period extended: {0}")]
    ValidityPeriodExtended(String),
    #[error("Permissions extended: {0}")]
    PermissionsExtended(String),
    #[error("Key usage extended: {0}")]
    KeyUsageExtended(String),
    #[error("Not permitted: {0}")]
    NotPermitted(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),
}
