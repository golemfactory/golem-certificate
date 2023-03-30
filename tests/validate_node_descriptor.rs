use golem_certificate::{
    schemas::permissions::{OutboundPermissions, PermissionDetails, Permissions},
    validator::{validate_node_descriptor_str, validated_data::ValidatedNodeDescriptor},
};
use url::Url;
use ya_client_model::NodeId;

#[test]
fn happy_path() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/node_descriptor_happy_path.signed.json").unwrap();

    let result = validate_node_descriptor_str(&node_descriptor).unwrap();

    assert_eq!(
        result,
        ValidatedNodeDescriptor {
            node_id: "0x338e02f29b63155beec8253af7ad367dd44b40c6"
                .parse::<NodeId>()
                .unwrap(),
            permissions: Permissions::Object(PermissionDetails {
                outbound: Some(OutboundPermissions::Urls(
                    [Url::parse("https://example.net/").unwrap()].into()
                ))
            }),
            certificate_chain_fingerprints: vec![
                "cb16a2ed213c1cf7e14faa7cf05743bc145b8555ec2eedb6b12ba0d31d17846d2ed4341b048f2e43b1ca5195a347bfeb0cd663c9e6002a4adb7cc7385112d3cc".into(),
                "80c84b2701126669966f46c1159cae89c58fb088e8bf94b318358fa4ca33ee56d8948511a397e5aba6aa5b88fff36f2541a91b133cde0fb816e8592b695c04c3".into(),
            ]
        }
    );
}

#[test]
fn error_when_node_signature_is_invalid() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/node_descriptor_invalid_signature.signed.json")
            .unwrap();

    let result = validate_node_descriptor_str(&node_descriptor);

    assert!(result.is_err());
}

#[test]
fn error_when_descriptor_is_expired() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/node_descriptor_expired.signed.json").unwrap();

    let result = validate_node_descriptor_str(&node_descriptor);

    assert!(result.is_err());
}
