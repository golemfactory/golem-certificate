use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use super::Constraints;

pub fn validate_constraints(
    parent: &Constraints,
    child: &Constraints,
    now: DateTime<Utc>,
) -> Result<()> {
    validate_not_before(&parent.not_before, &child.not_before)?;
    validate_not_after(&parent.not_after, &child.not_after)?;
    validate_current_time(child, now)?;

    Ok(())
}

fn validate_current_time(child: &Constraints, now: DateTime<Utc>) -> Result<()> {
    if now > child.not_after {
        Err(anyhow!("Child is not valid anymore"))
    } else if now < child.not_before {
        Err(anyhow!("Child is not valid yet"))
    } else {
        Ok(())
    }
}

fn validate_not_before(parent: &DateTime<Utc>, child: &DateTime<Utc>) -> Result<()> {
    if child < parent {
        Err(anyhow!(
            "Child 'not_before' property cannot be earlier than parent one"
        ))
    } else {
        Ok(())
    }
}

fn validate_not_after(parent: &DateTime<Utc>, child: &DateTime<Utc>) -> Result<()> {
    if child > parent {
        Err(anyhow!(
            "Child 'not_after' property cannot be later than parent one"
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod should {
    use utils::*;

    use super::*;

    #[test]
    fn accept_because_child_time_constraints_are_subset_of_parent() {
        let parent = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Constraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T02:02:02Z");

        assert!(validate_constraints(parent, child, now).is_ok());
    }

    #[test]
    fn accept_because_child_time_constraints_are_the_same_as_parent() {
        let parent = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let now = dt("2000-01-01T02:02:02Z");

        assert!(validate_constraints(parent, child, now).is_ok());
    }

    #[test]
    fn reject_because_child_has_earlier_not_before() {
        let parent = &Constraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T02:02:02Z");

        assert!(validate_constraints(parent, child, now).is_err());
    }

    #[test]
    fn reject_because_child_has_later_not_after() {
        let parent = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Constraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T05:05:05Z"),
        };
        let now = dt("2000-01-01T02:02:02Z");

        assert!(validate_constraints(parent, child, now).is_err());
    }

    #[test]
    fn reject_because_child_has_expired() {
        let parent = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Constraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T04:04:04Z");

        assert!(validate_constraints(parent, child, now).is_err());
    }

    #[test]
    fn reject_because_child_has_not_yet_valid() {
        let parent = &Constraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &Constraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T00:00:00Z");

        assert!(validate_constraints(parent, child, now).is_err());
    }

    mod utils {
        use super::*;

        pub fn dt(s: &str) -> DateTime<Utc> {
            s.parse().unwrap()
        }
    }
}
