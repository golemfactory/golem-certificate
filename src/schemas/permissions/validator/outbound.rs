use super::super::OutboundPermissions;

pub struct OutboundPermissionsExtendedError(());

pub fn validate_outbound_permissions(
    parent: &Option<OutboundPermissions>,
    child: &Option<OutboundPermissions>,
) -> Result<(), OutboundPermissionsExtendedError> {
    match (&parent, &child) {
        (_, None) => Ok(()),
        (None, Some(_)) => Err(OutboundPermissionsExtendedError(())),
        (Some(parent), Some(child)) => validate_url_permissions(parent, child),
    }
}

fn validate_url_permissions(
    parent: &OutboundPermissions,
    child: &OutboundPermissions,
) -> Result<(), OutboundPermissionsExtendedError> {
    match (parent, child) {
        (OutboundPermissions::Unrestricted, _) => Ok(()),
        (OutboundPermissions::Urls(_), OutboundPermissions::Unrestricted) => {
            Err(OutboundPermissionsExtendedError(()))
        }
        (OutboundPermissions::Urls(parent_urls), OutboundPermissions::Urls(child_urls)) => {
            if child_urls.is_subset(parent_urls) {
                Ok(())
            } else {
                Err(OutboundPermissionsExtendedError(()))
            }
        }
    }
}

#[cfg(test)]
mod should {
    use super::*;
    use utils::*;

    use test_case::test_case;
    use url::Url;

    #[test_case(OutboundPermissions::Unrestricted)]
    #[test_case(OutboundPermissions::Urls([Url::parse("https://1.net").unwrap()].into()))]
    fn accept_outbound_permissions_because_parent_has_unrestricted(child: OutboundPermissions) {
        let parent = Some(OutboundPermissions::Unrestricted);

        let child = Some(child);

        assert!(validate_outbound_permissions(&parent, &child).is_ok());
    }

    #[test]
    fn accept_outbound_permissions_because_child_does_not_want_outbound() {
        let parent = None;

        let child = Some(OutboundPermissions::Unrestricted);

        assert!(validate_outbound_permissions(&parent, &child).is_err());
    }

    #[test]
    fn reject_outbound_permissions_because_parent_has_no_outbound_permitted() {
        let parent = Some(OutboundPermissions::Unrestricted);

        let child = None;

        assert!(validate_outbound_permissions(&parent, &child).is_ok());
    }
    #[test]
    fn reject_outbound_permissions_because_parent_has_urls_and_child_has_unrestricted() {
        let parent = url_list_to_outbound_permissions(&["https://example.net"]);

        let child = Some(OutboundPermissions::Unrestricted);

        assert!(validate_outbound_permissions(&parent, &child).is_err());
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

        assert!(validate_outbound_permissions(&parent, &child).is_ok());
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

        assert!(validate_outbound_permissions(&parent, &child).is_err());
    }

    mod utils {
        use super::*;

        pub fn url_list_to_outbound_permissions(urls: &[&str]) -> Option<OutboundPermissions> {
            Some(OutboundPermissions::Urls(
                urls.iter()
                    .cloned()
                    .map(|u| Url::parse(u).unwrap())
                    .collect(),
            ))
        }
    }
}
