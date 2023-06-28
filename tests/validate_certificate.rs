use std::collections::HashSet;

use chrono::{DateTime, Utc};
use golem_certificate::{
    schemas::{
        certificate::key_usage::{KeyUsage, Usage},
        permissions::{OutboundPermissions, PermissionDetails, Permissions},
        subject::{Contact, Subject},
        validity_period::ValidityPeriod,
    },
    validator::{validate_certificate_str, validated_data::ValidatedCertificate},
    Error,
};
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test]
fn happy_path_details() {
    let certificate =
        std::fs::read_to_string("tests/resources/certificate/happy_path.signed.json").unwrap();

    let result = validate_certificate_str(&certificate, Some(Utc::now())).unwrap();

    assert_eq!(
        result,
        ValidatedCertificate {
            subject: Subject {
                display_name: "Example leaf cert".into(),
                contact: Contact { email: "example@leaf.tld".into(), additional_properties: Default::default() },
                additional_properties: Default::default(),
            },
            validity_period: ValidityPeriod {
                not_before: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
                 not_after: DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc)
            },
            certificate_chain_fingerprints: vec![
                "181eece864b9a8cd4ad661a967453db42ad01c0cb0e46bd2370ecec8f059797f3f369275e4a41a626907c9ea67641003a4b04e3506c65ee9e012232ea783c5d3".into(),
                "4f0c5b10741a8746141badf3b21325176a0e4e84dfe39747cb857b1c58dc65380ce85eb76a9986303f228a97a17012e77cc9e30ca595c077553309ade6cd2eb6".into(),
                "80c84b2701126669966f46c1159cae89c58fb088e8bf94b318358fa4ca33ee56d8948511a397e5aba6aa5b88fff36f2541a91b133cde0fb816e8592b695c04c3".into()
                ],
            permissions: Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Unrestricted) }),
            key_usage: KeyUsage::Limited(HashSet::from_iter(vec![Usage::SignNode].into_iter())),
        }
    );
}

#[test_case("happy_path.signed.json")]
#[test_case("happy_path_smartcard_root.signed.json")]
#[test_case("happy_path_smartcard_leaf.signed.json")]
fn happy_path(filename: &str) {
    let certificate =
        std::fs::read_to_string(format!("tests/resources/certificate/{filename}")).unwrap();

    assert!(validate_certificate_str(&certificate, Some(Utc::now())).is_ok());
}

#[test_case("not_signed.json", Error::JsonDoesNotConformToSchema("missing field `signature`".to_string()))]
#[test_case("expired.signed.json", Error::Expired("2023-01-02T00:00:00Z".parse().unwrap()))]
#[test_case("invalid_public_key.signed.json", Error::InvalidPublicKey)]
#[test_case("invalid_signature.signed.json", Error::InvalidSignature)]
#[test_case(
    "invalid_key_usage.signed.json",
   Error::KeyUsageExtended{parent: KeyUsage::Limited([Usage::SignNode, Usage::SignCertificate].into_iter().collect()), child: KeyUsage::Limited([Usage::SignNode, Usage::SignManifest].into_iter().collect())}
)]
#[test_case(
    "invalid_permissions.signed.json",
   Error::PermissionsExtended{parent: Permissions::Object(PermissionDetails{outbound: Some(OutboundPermissions::Unrestricted)}), child: Permissions::All}
)]
#[test_case("extended_validity_period.signed.json", Error::ValidityPeriodExtended{parent: ValidityPeriod{not_before: "2023-01-01T00:00:00Z".parse().unwrap(), not_after: "2025-01-01T00:00:00Z".parse().unwrap()}, child: ValidityPeriod{not_before: "2023-01-01T00:00:00Z".parse().unwrap(), not_after: "2099-01-01T00:00:00Z".parse().unwrap()}})]
#[test_case("cert_cannot_sign_other_cert.signed.json", Error::CertSignNotPermitted)]
fn should_return_err(filename: &str, expected_err: Error) {
    let certificate =
        std::fs::read_to_string(format!("tests/resources/certificate/{filename}")).unwrap();

    let result = validate_certificate_str(&certificate, Some(Utc::now()));

    assert_eq!(result.unwrap_err(), expected_err);
}
