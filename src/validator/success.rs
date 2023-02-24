use ya_client_model::NodeId;

use crate::schemas::permissions::Permissions;

use super::certificate_descriptor::CertificateId;

#[derive(Debug, PartialEq)]
pub enum Success {
    NodeDescriptor {
        node_id: NodeId,
        permissions: Permissions,
        certs: Vec<CertificateId>,
    },
    Certificate {
        permissions: Permissions,
        certs: Vec<CertificateId>,
    },
}
