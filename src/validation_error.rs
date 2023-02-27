use crate::schemas::{certificate::Fingerprint, validity_period::ValidityPeriod};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Validity period has expired: not_before: '{}' not_after: '{}' ", .0.not_before, .0.not_after)]
    Expired(ValidityPeriod),
    #[error("Certificate has invalid signature: '{0}'")]
    InvalidSignature(Fingerprint),
    #[error("Certificate does not have all required permissions: '{0}'")]
    PermissionsDoNotMatch(Fingerprint),
}
