use ya_client_model::NodeId;

use crate::schemas::{certificate::Fingerprint, permissions::Permissions};

#[derive(Debug, PartialEq)]
pub enum Success {
    NodeDescriptor {
        node_id: NodeId,
        permissions: Permissions,
        certs: Vec<Fingerprint>,
    },
    Certificate {
        permissions: Permissions,
        certs: Vec<Fingerprint>,
    },
}
