use crate::schemas::permissions::Permissions;

#[derive(Debug)]
pub struct CertificateId {
    pub public_key: String, // hex
    pub hash: String,       // hex
}

#[derive(Debug)]
pub struct CertificateDescriptor {
    id: CertificateId,
    permissions: Permissions,
}
