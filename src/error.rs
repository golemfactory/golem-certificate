use chrono::{DateTime, Utc};

use crate::schemas::{
    certificate::key_usage::KeyUsage,
    permissions::{OutboundPermissions, Permissions},
    validity_period::ValidityPeriod,
};

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("Expired: was valid to {0}")]
    Expired(DateTime<Utc>),
    #[error("Not valid yet: will be valid from {0}")]
    NotValidYet(DateTime<Utc>),
    #[error("Validity period extended: {parent:?}, {child:?}")]
    ValidityPeriodExtended {
        parent: ValidityPeriod,
        child: ValidityPeriod,
    },
    #[error("Permissions extended: {parent:?}, {child:?}")]
    PermissionsExtended {
        parent: Permissions,
        child: Permissions,
    },
    #[error("Outbound permissions extended: {parent:?}, {child:?}")]
    OutboundPermissionsExtended {
        parent: Option<OutboundPermissions>,
        child: Option<OutboundPermissions>,
    },
    #[error("Key usage extended: {parent:?}, {child:?}")]
    KeyUsageExtended { parent: KeyUsage, child: KeyUsage },
    #[error("Certificate signing not permitted")]
    CertSignNotPermitted,
    #[error("Certificate cannot sign Node Descriptor")]
    NodeSignNotPermitted,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Missing property: {0}")]
    MissingProperty(String),
    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
    #[error("Invalid json: {0}")]
    InvalidJson(String),
    #[error("Unsupported schema for structure {structure_name}: {schema}")]
    UnsupportedSchema {
        schema: String,
        structure_name: String,
    },
    #[error("Unsupported encryption algorithm: {0}")]
    UnsupportedEncryptionAlgorithm(String), //TODO Rafał
    #[error("Todo")]
    Todo, //TODO Rafał
}
