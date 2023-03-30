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
    #[error("TODO")]
    PermissionsExtended,
    #[error("TODO")]
    InvalidSignature,
    #[error("TODO")]
    InvalidSchema,
    #[error("TODO")]
    CertificateSigningNotPermitted,
    #[error("TODO")]
    NodeDescriptorSigningNotPermitted,
}
