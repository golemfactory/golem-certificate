use anyhow::{anyhow, Result};

use super::{KeyUsage, Usage};

pub fn validate_certificates_key_usage(parent: &KeyUsage, child: &KeyUsage) -> Result<()> {
    match (parent, child) {
        (KeyUsage::All, _) => Ok(()),
        (KeyUsage::Usages(_), KeyUsage::All) => Err(anyhow!(
            "Child cannot have 'All' key usage when parent doesn't have one"
        )),
        (KeyUsage::Usages(parent), KeyUsage::Usages(child)) => {
            if child.is_subset(parent) {
                Ok(())
            } else {
                Err(anyhow!("Child cannot extend key usages"))
            }
        }
    }
}

pub fn validate_sign_node(key_usage: &KeyUsage) -> Result<()> {
    match key_usage {
        KeyUsage::All => Ok(()),
        KeyUsage::Usages(usages) => {
            if usages.contains(&Usage::SignNode) {
                Ok(())
            } else {
                Err(anyhow!("Key usage does not allow to sign nodes"))
            }
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;

    use utils::*;

    use test_case::test_case;

    #[test_case(KeyUsage::All)]
    #[test_case(KeyUsage::Usages([Usage::SignNode].into()))]
    #[test_case(KeyUsage::Usages([Usage::SignManifest].into()))]
    #[test_case(KeyUsage::Usages([Usage::SignCertificate].into()))]
    fn accept_bacause_parent_has_all_permissions(child: KeyUsage) {
        let parent = KeyUsage::All;

        assert!(validate_certificates_key_usage(&parent, &child).is_ok());
    }

    #[test_case(KeyUsage::Usages([Usage::SignNode].into()))]
    #[test_case(KeyUsage::Usages([Usage::SignManifest].into()))]
    #[test_case(KeyUsage::Usages([Usage::SignCertificate].into()))]
    fn reject_bacause_child_requests_all_usage_and_parent_does_not_have_one(parent: KeyUsage) {
        let child = KeyUsage::All;

        assert!(validate_certificates_key_usage(&parent, &child).is_err());
    }

    #[test_case(&[], &[])]
    #[test_case(&[Usage::SignNode], &[])]
    #[test_case(&[Usage::SignNode], &[Usage::SignNode])]
    #[test_case(&[Usage::SignNode, Usage::SignCertificate], &[Usage::SignNode])]
    #[test_case(&[Usage::SignNode, Usage::SignCertificate, Usage::SignManifest], &[Usage::SignNode, Usage::SignCertificate, Usage::SignManifest])]
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

    #[test_case(&[Usage::SignNode])]
    #[test_case(&[Usage::SignNode, Usage::SignManifest])]
    #[test_case(&[Usage::SignNode, Usage::SignCertificate])]
    #[test_case(&[Usage::SignNode, Usage::SignCertificate, Usage::SignManifest])]
    fn accept_sign_node_validation_because_cert_has_proper_usage(key_usage: &[Usage]) {
        let key_usage = slice_to_usages(key_usage);

        assert!(validate_sign_node(&key_usage).is_ok());
    }

    #[test]
    fn accept_sign_node_validation_because_cert_has_all_usage() {
        let key_usage = KeyUsage::All;

        assert!(validate_sign_node(&key_usage).is_ok());
    }

    #[test_case(&[])]
    #[test_case(&[Usage::SignManifest])]
    #[test_case(&[Usage::SignCertificate, Usage::SignManifest])]
    fn reject_sign_node_validation_because_cert_has_no_proper_usage(key_usage: &[Usage]) {
        let key_usage = slice_to_usages(key_usage);

        assert!(validate_sign_node(&key_usage).is_err());
    }
    mod utils {
        use super::*;

        use crate::key_usage::KeyUsage;

        pub fn slice_to_usages(s: &[Usage]) -> KeyUsage {
            KeyUsage::Usages(s.iter().cloned().collect())
        }
    }
}
