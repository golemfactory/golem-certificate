use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use super::Validity;

//TODO Rafał rename validity to constraints
pub fn validate_validity(parent: &Validity, child: &Validity) -> Result<()> {
    validate_not_before(&parent.not_before, &child.not_before)?;

    Ok(())
}

fn validate_not_before(parent: &DateTime<Utc>, child: &DateTime<Utc>) -> Result<()> {
    if child >= parent {
        Ok(())
    } else {
        Err(anyhow!(
            "Child 'not_before' property cannot be earlier than parent one"
        ))
    }
}

#[cfg(test)]
mod should {
    use utils::*;

    use super::*;

    #[test]
    fn accept_because_child_time_constraints_are_subset_of_parent() {
        let parent = &Validity {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Validity {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };

        assert!(validate_validity(parent, child).is_ok());
    }

    #[test]
    fn accept_because_child_time_constraints_are_the_same_as_parent() {
        let parent = &Validity {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Validity {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };

        assert!(validate_validity(parent, child).is_ok());
    }

    #[test]
    fn reject_because_child_has_earlier_not_before() {
        let parent = &Validity {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Validity {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };

        assert!(validate_validity(parent, child).is_err());
    }

    mod utils {
        use super::*;

        pub fn dt(s: &str) -> DateTime<Utc> {
            s.parse().unwrap()
        }
    }
}
