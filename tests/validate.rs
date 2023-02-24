use golem_certificate::{
    schemas::permissions::{OutboundPermissions, PermissionDetails, Permissions},
    validator::{certificate_descriptor::CertificateId, success::Success, validate},
};
use url::Url;
use ya_client_model::NodeId;

#[test]
fn happy_path() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/happy_path_node_descriptor.json").unwrap();

    let result = validate(&node_descriptor).unwrap();

    assert_eq!(
        result,
        Success::NodeDescriptor {
            node_id: "0xbabe000000000000000000000000000000000000"
                .parse::<NodeId>()
                .unwrap(),
            permissions: Permissions::Object(PermissionDetails {
                outbound: Some(OutboundPermissions::Urls(
                    [Url::parse("https://example.net/").unwrap()].into()
                ))
            }),
            certs: vec![
                CertificateId {
                    fingerprint: "mock_fingerprint".into(),
                },
                CertificateId {
                    fingerprint: "mock_fingerprint".into(),
                }
            ]
        }
    );
}
