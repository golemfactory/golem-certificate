use super::certificate_descriptor::CertificateId;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid data")]
    InvalidData,
    #[error("Certificate is expired: '{}' ", .0.hash)]
    Expired(CertificateId),
    #[error("Certificate has invalid signature: '{}'", .0.hash)]
    InvalidSignature(CertificateId),
    #[error("Certificate does not have all required permissions: '{}'", .0.hash)]
    PermissionsDoNotMatch(CertificateId),
    #[error("Url parse error {0:?}")]
    UrlParseError(Vec<String>),
}
