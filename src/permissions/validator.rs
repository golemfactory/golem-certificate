use anyhow::{anyhow, Result};

use super::{OutboundPermissions, Permissions};

pub fn validate_permissions(parent: &Permissions, child: &Permissions) -> Result<()> {
    match (parent, child) {
        (Permissions::All, _) => Ok(()),
        (Permissions::Object { .. }, Permissions::All) => Err(anyhow!(
            "Child cannot have 'All' permissions when parent doesn't have one"
        )),
        (Permissions::Object(parent), Permissions::Object(child)) => {
            validate_outbound_permissions(&parent.outbound, &child.outbound)
        }
    }
}

fn validate_outbound_permissions(
    parent: &Option<OutboundPermissions>,
    child: &Option<OutboundPermissions>,
) -> Result<()> {
    match (&parent, &child) {
        (_, None) => Ok(()),
        (None, Some(_)) => Err(anyhow!(
            "Child wants to have outbound permissions, but parent doesn't have ones"
        )),
        (Some(parent), Some(child)) => match (parent, child) {
            (OutboundPermissions::Unrestricted, _) => Ok(()),
            (OutboundPermissions::Urls(_), OutboundPermissions::Unrestricted) => {
                Err(anyhow!("Child cannot extend outbound permissions"))
            }
            (OutboundPermissions::Urls(parent_urls), OutboundPermissions::Urls(child_urls)) => {
                if child_urls.is_subset(parent_urls) {
                    Ok(())
                } else {
                    Err(anyhow!("Child cannot extend outbound permitted urls"))
                }
            }
        },
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use utils::*;

    use crate::permissions::PermissionDetails;

    use test_case::test_case;
    use url::Url;

    #[test_case(Permissions::All)]
    #[test_case(Permissions::Object(PermissionDetails {outbound: Some(OutboundPermissions::Unrestricted)}))]
    #[test_case(Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into()))}))]
    fn accept_because_parent_has_all_permissions(child: Permissions) {
        let parent = Permissions::All;

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test_case(Permissions::Object(PermissionDetails {outbound: Some(OutboundPermissions::Unrestricted)}))]
    #[test_case(Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into()))}))]
    fn reject_because_child_requests_all_permissions_and_parent_does_not_have_one(
        parent: Permissions,
    ) {
        let child = Permissions::All;

        assert!(validate_permissions(&parent, &child).is_err());
    }

    #[test_case(OutboundPermissions::Unrestricted)]
    #[test_case(OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into()))]
    fn accept_outbound_permissions_because_parent_has_unrestricted(child: OutboundPermissions) {
        let parent = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });

        let child = Permissions::Object(PermissionDetails {
            outbound: Some(child),
        });

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test]
    fn accept_outbound_permissions_because_child_does_not_want_outbound() {
        let parent = Permissions::Object(PermissionDetails { outbound: None });

        let child = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });

        assert!(validate_permissions(&parent, &child).is_err());
    }

    #[test]
    fn reject_outbound_permissions_because_parent_has_no_outbound_permitted() {
        let parent = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });

        let child = Permissions::Object(PermissionDetails { outbound: None });

        assert!(validate_permissions(&parent, &child).is_ok());
    }
    #[test]
    fn reject_outbound_permissions_because_parent_has_urls_and_child_has_unrestricted() {
        let parent = url_list_to_outbound_permissions(&["https://example.net"]);

        let child = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });

        assert!(validate_permissions(&parent, &child).is_err());
    }

    #[test_case(&[], &[])]
    #[test_case(&["https://1.net"], &[])]
    #[test_case(&["https://1.net"], &["https://1.net"])]
    #[test_case(&["https://1.net", "https://2.net"], &["https://1.net"])]
    fn accept_outbound_permissions_because_child_urls_are_subset_of_parent_ones(
        parent_urls: &[&str],
        child_urls: &[&str],
    ) {
        let parent = url_list_to_outbound_permissions(parent_urls);
        let child = url_list_to_outbound_permissions(child_urls);

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test_case(&[], &["https://xxx.net"])]
    #[test_case(&["https://1.net"], &["https://xxx.net"])]
    #[test_case(&["https://1.net"], &["https://1.net", "https://xxx.net"])]
    fn reject_outbound_permissions_because_child_urls_are_not_a_subset_of_parent_ones(
        parent_urls: &[&str],
        child_urls: &[&str],
    ) {
        let parent = url_list_to_outbound_permissions(parent_urls);
        let child = url_list_to_outbound_permissions(child_urls);

        assert!(validate_permissions(&parent, &child).is_err());
    }

    mod utils {
        use super::*;

        pub fn url_list_to_outbound_permissions(urls: &[&str]) -> Permissions {
            Permissions::Object(PermissionDetails {
                outbound: Some(OutboundPermissions::Urls(
                    urls.iter()
                        .cloned()
                        .map(|u| Url::parse(u).unwrap())
                        .collect(),
                )),
            })
        }
    }
}
