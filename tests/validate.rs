use golem_certificate::{
    schemas::{
        node_descriptor::NodeDescriptor,
        permissions::{OutboundPermissions, PermissionDetails, Permissions},
        validity_period::ValidityPeriod,
    },
    validator::{
        validate_node_descriptor,
        validated_data::{ValidatedData, ValidatedNodeDescriptor},
    },
};
use url::Url;
use ya_client_model::NodeId;

#[test]
fn happy_path() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/happy_path_node_descriptor.json").unwrap();

    let result = validate_node_descriptor(&node_descriptor).unwrap();

    assert_eq!(
        result,
        ValidatedNodeDescriptor {
            descriptor: NodeDescriptor {
                node_id: "0xbabe000000000000000000000000000000000000"
                    .parse::<NodeId>()
                    .unwrap(),
                validity_period: ValidityPeriod {
                    not_before: "2000-01-01T00:00:00Z".parse().unwrap(),
                    not_after: "2030-01-01T00:00:00Z".parse().unwrap()
                },
                permissions: Permissions::Object(PermissionDetails {
                    outbound: Some(OutboundPermissions::Urls(
                        [Url::parse("https://example.net/").unwrap()].into()
                    ))
                })
            },
            chain: vec!["mock_fingerprint".into(), "mock_fingerprint".into(),]
        }
    );
}
