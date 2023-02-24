use golem_certificate::{
    schemas::permissions::{OutboundPermissions, PermissionDetails, Permissions},
    validator::{certificate_descriptor::CertificateId, success::Success, validate},
};
use url::Url;

#[test]
fn happy_path() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/happy_path_node_descriptor.json").unwrap();

    let result = validate(&node_descriptor).unwrap();

    assert_eq!(
        result,
        Success::NodeDescriptor {
            node_id: "0xbabe000000000000000000000000000000000000"
                .parse::<String>()
                .unwrap(),
            permissions: Permissions::Object(PermissionDetails {
                outbound: Some(OutboundPermissions::Urls(
                    [Url::parse("https://example.net/").unwrap()].into()
                ))
            }),
            certs: vec![
                CertificateId {
                    public_key: "public key todo".into(),
                    hash: "hash todo".into()
                },
                CertificateId {
                    public_key: "public key todo".into(),
                    hash: "hash todo".into()
                }
            ]
        }
    );
}
