use std::collections::HashSet;

use anyhow::{anyhow, Result};

use super::Permission;

pub fn validate_permissions(parent: &[Permission], child: &[Permission]) -> Result<()> {
    let mut permitted_urls = HashSet::new();

    for parent_perm in parent {
        match parent_perm {
            Permission::Outbound(urls) => permitted_urls.extend(urls),
            _ => {}
        }
    }

    for child_perm in child {
        match child_perm {
            Permission::All => {
                return Err(anyhow!("All permission cannot be passed down a chain"));
            }
            Permission::Outbound(urls) => {
                let urls: HashSet<_> = urls.iter().collect();
                if !urls.is_subset(&permitted_urls) && !parent.contains(&Permission::All) {
                    return Err(anyhow!("Permitted urls cannot be extended"));
                }
            }
            Permission::OutboundUnrestricted => {
                if !parent.contains(&Permission::OutboundUnrestricted)
                    && !parent.contains(&Permission::All)
                {
                    return Err(anyhow!("Permissions cannot be extended"));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod should {
    use super::*;

    use test_case::test_case;
    use url::Url;

    #[test_case(&[], &[])]
    #[test_case(&[Permission::OutboundUnrestricted], &[])]
    #[test_case(&[Permission::All], &[])]
    #[test_case(&[Permission::OutboundUnrestricted], &[Permission::OutboundUnrestricted])]
    #[test_case(&[Permission::Outbound(vec![])], &[Permission::Outbound(vec![])])]
    #[test_case(&[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()])], &[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()])])]
    fn be_valid_because_child_is_subset_of_parent(parent: &[Permission], child: &[Permission]) {
        assert!(validate_permissions(parent, child).is_ok());
    }

    #[test_case(&[Permission::All], &[Permission::OutboundUnrestricted])]
    #[test_case(&[Permission::All], &[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()])])]
    fn be_valid_because_parent_has_all_permissions(parent: &[Permission], child: &[Permission]) {
        assert!(validate_permissions(parent, child).is_ok());
    }

    #[test_case(&[Permission::Outbound(vec![])], &[Permission::Outbound(vec![])])]
    #[test_case(&[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()])], &[Permission::Outbound(vec![])])]
    #[test_case(&[Permission::Outbound(vec![Url::parse("https://1.net").unwrap(), Url::parse("https://2.net").unwrap()])], &[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()])])]
    #[test_case(&[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()]), Permission::Outbound(vec![Url::parse("https://2.net").unwrap()])], &[Permission::Outbound(vec![Url::parse("https://1.net").unwrap(), Url::parse("https://2.net").unwrap()])])]
    fn be_valid_because_child_outbound_url_list_is_subset_of_parent(
        parent: &[Permission],
        child: &[Permission],
    ) {
        assert!(validate_permissions(parent, child).is_ok());
    }

    #[test_case(&[Permission::All], &[Permission::All])]
    #[test_case(&[Permission::All, Permission::OutboundUnrestricted], &[Permission::All])]
    fn be_invalid_because_child_cannot_have_all_permission(
        parent: &[Permission],
        child: &[Permission],
    ) {
        assert!(validate_permissions(parent, child).is_err());
    }

    #[test_case(&[Permission::Outbound(vec![Url::parse("https://1.net").unwrap()])], &[Permission::OutboundUnrestricted])]
    fn be_invalid_because_child_permissions_must_be_superset_of_parent(
        parent: &[Permission],
        child: &[Permission],
    ) {
        assert!(validate_permissions(parent, child).is_err());
    }

    #[test_case(&[Permission::Outbound(vec![Url::parse("https://1.net").unwrap(), Url::parse("https://2.net").unwrap()])], &[Permission::Outbound(vec![Url::parse("https://1.net").unwrap(), Url::parse("https://3.net").unwrap()])])]
    fn be_invalid_because_child_outbound_url_list_is_not_subset_of_parent(
        parent: &[Permission],
        child: &[Permission],
    ) {
        assert!(validate_permissions(parent, child).is_err());
    }
}
