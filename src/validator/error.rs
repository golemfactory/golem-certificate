use crate::schemas::certificate::Fingerprint;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid data")]
    InvalidData,
    #[error("Certificate is expired: '{0}' ")]
    Expired(Fingerprint),
    #[error("Certificate has invalid signature: '{0}'")]
    InvalidSignature(Fingerprint),
    #[error("Certificate does not have all required permissions: '{0}'")]
    PermissionsDoNotMatch(Fingerprint),
    #[error("Url parse error {0:?}")]
    UrlParseError(Vec<String>),
}
