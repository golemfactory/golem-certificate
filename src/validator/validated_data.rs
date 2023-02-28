use ya_client_model::NodeId;

use crate::schemas::{
    certificate::{Certificate, Fingerprint},
    node_descriptor::NodeDescriptor,
    permissions::Permissions,
};

#[derive(Debug, PartialEq)]
pub enum ValidatedData {
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

#[derive(Debug, PartialEq)]
pub struct ValidatedCert {
    pub cert: Certificate,
    pub chain: Vec<Fingerprint>,
}

#[derive(Debug, PartialEq)]
pub struct ValidatedNodeDescriptor {
    pub descriptor: NodeDescriptor,
    pub chain: Vec<Fingerprint>,
}
