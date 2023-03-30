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

#[test]
fn happy_path() {
    let node_descriptor =
        std::fs::read_to_string("tests/resources/certificates/happy_path_certificate.signed.json")
            .unwrap();

    let result = validate_certificate_str(&node_descriptor).unwrap();

    assert_eq!(
        result,
        ValidatedCertificate {
            subject: Subject {
                display_name: "Example partner cert".into(),
                contact: Contact { email: "example@partner.tld".into(), additional_properties: Default::default() },
                additional_properties: Default::default(),
            },
            validity_period: ValidityPeriod {
                not_before: DateTime::parse_from_rfc3339("2023-01-01T00:00:00Z").unwrap().with_timezone(&Utc),
                 not_after: DateTime::parse_from_rfc3339("2025-01-01T00:00:00Z").unwrap().with_timezone(&Utc)
            },
            certificate_chain_fingerprints: vec![
                "cb16a2ed213c1cf7e14faa7cf05743bc145b8555ec2eedb6b12ba0d31d17846d2ed4341b048f2e43b1ca5195a347bfeb0cd663c9e6002a4adb7cc7385112d3cc".into(), 
                "80c84b2701126669966f46c1159cae89c58fb088e8bf94b318358fa4ca33ee56d8948511a397e5aba6aa5b88fff36f2541a91b133cde0fb816e8592b695c04c3".into()
                ],
            permissions: Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Unrestricted) }),
            key_usage: KeyUsage::Limited(HashSet::from_iter(vec![Usage::SignNode].into_iter())),
        }
    );
}
