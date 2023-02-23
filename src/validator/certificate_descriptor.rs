#[derive(Debug)]
pub struct CertificateId {
    pub public_key: String, // hex
    pub hash: String,       // hex
}
