use crate::schemas::permissions::Permissions;

use super::certificate_descriptor::CertificateId;

#[derive(Debug, PartialEq)]
pub enum Success {
    NodeDescriptor {
        node_id: String,
        permissions: Permissions,
        certs: Vec<CertificateId>,
    },
    Certificate {
        permissions: Permissions,
        certs: Vec<CertificateId>,
    },
}
