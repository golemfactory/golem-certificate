use ya_client_model::NodeId;

use crate::schemas::{
    certificate::{key_usage::KeyUsage, Fingerprint},
    permissions::Permissions,
    subject::Subject,
    validity_period::ValidityPeriod,
};

#[derive(Debug, PartialEq)]
pub struct ValidatedNodeDescriptor {
    pub certificate_chain_fingerprints: Vec<Fingerprint>,
    pub permissions: Permissions,
    pub node_id: NodeId,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValidatedCertificate {
    pub certificate_chain_fingerprints: Vec<Fingerprint>,
    pub permissions: Permissions,
    pub key_usage: KeyUsage,
    pub validity_period: ValidityPeriod,
    pub subject: Subject,
}
