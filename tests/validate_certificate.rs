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
};
use pretty_assertions::assert_eq;
use test_case::test_case;

#[test]
fn happy_path() {
    let certificate =
        std::fs::read_to_string("tests/resources/certificate/happy_path.signed.json").unwrap();

    let result = validate_certificate_str(&certificate).unwrap();

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

#[test_case("not_signed.json")]
#[test_case("expired.signed.json")]
#[test_case("invalid_signature.signed.json")]
#[test_case("invalid_key_usage.signed.json")]
#[test_case("invalid_permissions.signed.json")]
#[test_case("extended_validity_period.signed.json")]
fn should_return_err(filename: &str) {
    let certificate =
        std::fs::read_to_string(format!("tests/resources/certificate/{filename}")).unwrap();

    let result = validate_certificate_str(&certificate);

    assert!(result.is_err());
}
