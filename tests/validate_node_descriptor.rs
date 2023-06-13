use chrono::Utc;
use golem_certificate::{
    schemas::permissions::{OutboundPermissions, PermissionDetails, Permissions},
    validator::{validate_node_descriptor_str, validated_data::ValidatedNodeDescriptor},
    Error,
};
use test_case::test_case;
use url::Url;
use ya_client_model::NodeId;

#[test]
fn happy_path() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/node_descriptor/happy_path.signed.json").unwrap();

    let result = validate_node_descriptor_str(&node_descriptor, Some(Utc::now())).unwrap();

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

#[test_case("not_signed.json", Error::JsonDoesNotConformToSchema("missing field `signature`".to_string()))]
#[test_case("invalid_signature.signed.json", Error::InvalidSignature)]
#[test_case("expired.signed.json", Error::Expired("2023-01-02T00:00:00Z".parse().unwrap()))]
#[test_case("invalid_permissions_chain.signed.json",
   Error::PermissionsExtended{parent: Permissions::Object(PermissionDetails{outbound: Some(OutboundPermissions::Unrestricted)}), child: Permissions::All}
)]
#[test_case("invalid_cert_chain_signature.signed.json", Error::InvalidSignature)]
#[test_case("cert_cannot_sign_node.signed.json", Error::NodeSignNotPermitted)]
fn should_return_err(filename: &str, expected_err: Error) {
    let node_descriptor =
        std::fs::read_to_string(format!("tests/resources/node_descriptor/{filename}")).unwrap();

    let result = validate_node_descriptor_str(&node_descriptor, Some(Utc::now()));

    assert_eq!(result.unwrap_err(), expected_err);
}
