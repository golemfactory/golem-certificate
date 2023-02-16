use anyhow::{anyhow, Result};

use super::{OutboundPermissions, Permissions};

pub fn validate_permissions(parent: &Permissions, child: &Permissions) -> Result<()> {
    match (parent, child) {
        (Permissions::All, _) => Ok(()),
        (Permissions::Object { outbound: _ }, Permissions::All) => Err(anyhow!(
            "Child cannot have 'All' permissions when parent doesn't have one"
        )),
        (
            Permissions::Object { outbound },
            Permissions::Object {
                outbound: child_outbound,
            },
        ) => todo!(),
    }
}

#[cfg(test)]
mod should {
    use super::*;

    use test_case::test_case;
    use url::Url;

    #[test_case(Permissions::All)]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Unrestricted})]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Urls(vec![Url::parse("https://1.net").unwrap()])})]
    fn accept_because_parent_has_all_permissions(child: Permissions) {
        let parent = Permissions::All;

        assert!(validate_permissions(&parent, &child).is_ok());
    }

    #[test_case(Permissions::Object{outbound: OutboundPermissions::Unrestricted})]
    #[test_case(Permissions::Object{outbound: OutboundPermissions::Urls(vec![Url::parse("https://1.net").unwrap()])})]
    fn reject_because_child_requests_all_permissions_and_parent_does_not_have_one(
        parent: Permissions,
    ) {
        let child = Permissions::All;
        assert!(validate_permissions(&parent, &child).is_err());
    }
}
