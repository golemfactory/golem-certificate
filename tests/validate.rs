use golem_certificate::{
    schemas::permissions::{OutboundPermissions, PermissionDetails, Permissions},
    validator::{validate_node_descriptor, validated_data::ValidatedNodeDescriptor},
};
use url::Url;
use ya_client_model::NodeId;

#[test]
fn happy_path() {
    // FIXME

    let node_descriptor =
        std::fs::read_to_string("tests/resources/happy_path_node_descriptor.json").unwrap();

    let result = validate_node_descriptor(&node_descriptor).unwrap();

    assert_eq!(
        result,
        ValidatedNodeDescriptor {
            node_id: "0xbabe000000000000000000000000000000000000"
                .parse::<NodeId>()
                .unwrap(),
            permissions: Permissions::Object(PermissionDetails {
                outbound: Some(OutboundPermissions::Urls(
                    [Url::parse("https://example.net/").unwrap()].into()
                ))
            }),
            certificate_chain_fingerprints: vec!["mock_fingerprint".into(), "mock_fingerprint".into(),]
        }
    );
}
