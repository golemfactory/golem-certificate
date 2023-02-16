use anyhow::{anyhow, Result};

use super::{OutboundPermissions, Permissions};

pub fn validate_permissions(parent: &Permissions, child: &Permissions) -> Result<()> {
    match (parent, child) {
        (Permissions::All, _) => Ok(()),
        (Permissions::Object { outbound: _ }, Permissions::All) => Err(anyhow!(
            "Child cannot have 'All' permissions when parent doesn't have one"
        )),
        (
            Permissions::Object {
                outbound: parent_outbound,
            },
            Permissions::Object {
                outbound: child_outbound,
            },
        ) => validate_outbound_permissions(parent_outbound, child_outbound),
    }
}

fn validate_outbound_permissions(
    parent: &OutboundPermissions,
    child: &OutboundPermissions,
) -> Result<()> {
    match (parent, child) {
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
    }
}

#[cfg(test)]
mod should {
    use super::*;

    use test_case::test_case;
    use url::Url;

    #[test_case(Permissions::All)]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Unrestricted})]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into())})]
    fn accept_because_parent_has_all_permissions(child: Permissions) {
        let parent = Permissions::All;

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test_case(Permissions::Object{outbound: OutboundPermissions::Unrestricted})]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into())})]
    fn reject_because_child_requests_all_permissions_and_parent_does_not_have_one(
        parent: Permissions,
    ) {
        let child = Permissions::All;
        assert!(validate_permissions(&parent, &child).is_err());
    }

    #[test_case(Permissions::Object{outbound: OutboundPermissions::Unrestricted})]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into())})]
    fn accept_outbound_permissions_because_parent_has_unrestricted(child: Permissions) {
        let parent = Permissions::Object {
            outbound: OutboundPermissions::Unrestricted,
        };

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test]
    fn reject_outbound_permissions_because_parent_has_urls_and_child_has_unrestricted() {
        let parent = Permissions::Object {
            outbound: OutboundPermissions::Urls(
                [Url::parse("https://example.net").unwrap()].into(),
            ),
        };

        let child = Permissions::Object {
            outbound: OutboundPermissions::Unrestricted,
        };
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
        let parent = Permissions::Object {
            outbound: OutboundPermissions::Urls(
                parent_urls
                    .iter()
                    .cloned()
                    .map(|u| Url::parse(u).unwrap())
                    .collect(),
            ),
        };
        let child = Permissions::Object {
            outbound: OutboundPermissions::Urls(
                child_urls
                    .iter()
                    .cloned()
                    .map(|u| Url::parse(u).unwrap())
                    .collect(),
            ),
        };

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test_case(&[], &["https://xxx.net"])]
    #[test_case(&["https://1.net"], &["https://xxx.net"])]
    #[test_case(&["https://1.net"], &["https://1.net", "https://xxx.net"])]
    fn reject_outbound_permissions_because_child_urls_are_not_a_subset_of_parent_ones(
        parent_urls: &[&str],
        child_urls: &[&str],
    ) {
        let parent = Permissions::Object {
            outbound: OutboundPermissions::Urls(
                parent_urls
                    .iter()
                    .cloned()
                    .map(|u| Url::parse(u).unwrap())
                    .collect(),
            ),
        };
        let child = Permissions::Object {
            outbound: OutboundPermissions::Urls(
                child_urls
                    .iter()
                    .cloned()
                    .map(|u| Url::parse(u).unwrap())
                    .collect(),
            ),
        };

        assert!(validate_permissions(&parent, &child).is_err());
    }
}
