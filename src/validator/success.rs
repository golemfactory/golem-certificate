use crate::schemas::permissions::Permissions;

use super::certificate_descriptor::CertificateId;

pub enum Success {
    NodeDescriptor {
        node_id: String,
        permissions: Permissions,
        certs: Vec<CertificateId>,
    },
    Certificate {
        //TODO Rafał What is usecase of validation of certificates? permissions seems not valid here
        permissions: Permissions,
        certs: Vec<CertificateId>,
    },
}
