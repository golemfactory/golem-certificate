use super::certificate_descriptor::CertificateDescriptor;

pub enum Success {
    NodeDescriptor {
        node_id: String,
        certs: Vec<CertificateDescriptor>,
    },
    Certificate {
        certs: Vec<CertificateDescriptor>,
    },
}
