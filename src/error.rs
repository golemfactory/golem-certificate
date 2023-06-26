use chrono::{DateTime, Utc};

use crate::schemas::{
    certificate::key_usage::KeyUsage, permissions::Permissions, validity_period::ValidityPeriod,
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
    #[error("Key usage extended: {parent:?}, {child:?}")]
    KeyUsageExtended { parent: KeyUsage, child: KeyUsage },
    #[error("Certificate signing not permitted")]
    CertSignNotPermitted,
    #[error("Certificate cannot sign Node Descriptor")]
    NodeSignNotPermitted,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid signature value (cannot deserialize)")]
    InvalidSignatureValue,
    #[error("Invalid public key")]
    InvalidPublicKey,
    #[error("Invalid json: {0}")]
    InvalidJson(String),
    #[error("JCS serialization error: {0}")]
    JcsSerializationError(String),
    #[error("Json does not conform to schema: {0}")]
    JsonDoesNotConformToSchema(String),
    #[error("Unsupported schema for structure {structure_name}: {schema}")]
    UnsupportedSchema {
        schema: String,
        structure_name: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
