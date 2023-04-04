use super::{KeyUsage, Usage};
use crate::Error;

pub fn validate_certificates_key_usage(parent: &KeyUsage, child: &KeyUsage) -> Result<(), Error> {
    match (parent, child) {
        (KeyUsage::All, _) => Ok(()),
        (KeyUsage::Limited(_), KeyUsage::All) => Err(Error::KeyUsageExtended {
            parent: parent.to_owned(),
            child: child.to_owned(),
        }),
        (KeyUsage::Limited(parent_limited), KeyUsage::Limited(child_limited)) => {
            if child_limited.is_subset(parent_limited) {
                if parent_limited.contains(&Usage::SignCertificate) {
                    Ok(())
                } else {
                    Err(Error::CertSignNotPermitted)
                }
            } else {
                Err(Error::KeyUsageExtended {
                    parent: parent.to_owned(),
                    child: child.to_owned(),
                })
            }
        }
    }
}

pub fn validate_sign_node(key_usage: &KeyUsage) -> Result<(), Error> {
    match key_usage {
        KeyUsage::All => Ok(()),
        KeyUsage::Limited(usages) => {
            if usages.contains(&Usage::SignNode) {
                Ok(())
            } else {
                Err(Error::NodeSignNotPermitted)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use utils::*;

    mod validate_certs_key_usage_should {
        use super::*;

        use test_case::test_case;

        #[test_case(KeyUsage::All)]
        #[test_case(KeyUsage::Limited([Usage::SignNode].into()))]
        #[test_case(KeyUsage::Limited([Usage::SignManifest].into()))]
        #[test_case(KeyUsage::Limited([Usage::SignCertificate].into()))]
        fn accept_bacause_parent_has_all_permissions(child: KeyUsage) {
            let parent = KeyUsage::All;

            assert!(validate_certificates_key_usage(&parent, &child).is_ok());
        }

        #[test_case(KeyUsage::Limited([Usage::SignNode].into()))]
        #[test_case(KeyUsage::Limited([Usage::SignManifest].into()))]
        #[test_case(KeyUsage::Limited([Usage::SignCertificate].into()))]
        fn reject_bacause_child_requests_all_usage_and_parent_does_not_have_one(parent: KeyUsage) {
            let child = KeyUsage::All;

            assert!(validate_certificates_key_usage(&parent, &child).is_err());
        }

        #[test_case(&[Usage::SignCertificate], &[])]
        #[test_case(&[Usage::SignCertificate], &[Usage::SignCertificate])]
        #[test_case(&[Usage::SignCertificate, Usage::SignNode], &[Usage::SignNode])]
        #[test_case(&[Usage::SignCertificate, Usage::SignNode], &[Usage::SignCertificate, Usage::SignNode])]
        #[test_case(&[Usage::SignCertificate, Usage::SignNode, Usage::SignManifest], &[Usage::SignNode, Usage::SignCertificate, Usage::SignManifest])]
        fn accept_because_child_usages_are_subset_of_parent(parent: &[Usage], child: &[Usage]) {
            let parent = slice_to_usages(parent);
            let child = slice_to_usages(child);

            assert!(validate_certificates_key_usage(&parent, &child).is_ok());
        }

        #[test_case(&[], &[Usage::SignNode])]
        #[test_case(&[Usage::SignCertificate], &[Usage::SignNode])]
        #[test_case(&[Usage::SignCertificate], &[Usage::SignNode, Usage::SignCertificate])]
        fn reject_because_child_usages_are_not_subset_of_parent(parent: &[Usage], child: &[Usage]) {
            let parent = slice_to_usages(parent);
            let child = slice_to_usages(child);

            assert!(validate_certificates_key_usage(&parent, &child).is_err());
        }

        #[test_case(&[Usage::SignNode], &[Usage::SignNode])]
        #[test_case(&[Usage::SignManifest], &[Usage::SignManifest])]
        fn reject_because_parent_cannot_sign_certs(parent: &[Usage], child: &[Usage]) {
            let parent = slice_to_usages(parent);
            let child = slice_to_usages(child);

            assert!(validate_certificates_key_usage(&parent, &child).is_err());
        }
    }

    mod validate_sign_node_should {
        use super::*;

        use test_case::test_case;

        #[test_case(&[Usage::SignNode])]
        #[test_case(&[Usage::SignNode, Usage::SignManifest])]
        #[test_case(&[Usage::SignNode, Usage::SignCertificate])]
        #[test_case(&[Usage::SignNode, Usage::SignCertificate, Usage::SignManifest])]
        fn accept_because_cert_has_proper_usage(key_usage: &[Usage]) {
            let key_usage = slice_to_usages(key_usage);

            assert!(validate_sign_node(&key_usage).is_ok());
        }

        #[test]
        fn accept_because_cert_has_all_usage() {
            let key_usage = KeyUsage::All;

            assert!(validate_sign_node(&key_usage).is_ok());
        }

        #[test_case(&[])]
        #[test_case(&[Usage::SignManifest])]
        #[test_case(&[Usage::SignCertificate, Usage::SignManifest])]
        fn reject_because_cert_has_no_proper_usage(key_usage: &[Usage]) {
            let key_usage = slice_to_usages(key_usage);

            assert!(validate_sign_node(&key_usage).is_err());
        }
    }

    mod utils {
        use super::*;

        pub fn slice_to_usages(s: &[Usage]) -> KeyUsage {
            KeyUsage::Limited(s.iter().cloned().collect())
        }
    }
}
