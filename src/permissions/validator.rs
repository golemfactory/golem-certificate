use anyhow::{anyhow, Result};

use super::Permissions;

mod outbound;
use outbound::validate_outbound_permissions;

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

#[cfg(test)]
mod should {
    use super::*;

    use crate::permissions::{OutboundPermissions, PermissionDetails};

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

    #[test]
    fn accept_valid_outbound_permissions() {
        let parent = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Unrestricted),
        });

        let child = Permissions::Object(PermissionDetails {
            outbound: Some(OutboundPermissions::Urls(
                [Url::parse("https://1.net").unwrap()].into(),
            )),
        });

        assert!(validate_permissions(&parent, &child).is_ok());
    }
}
