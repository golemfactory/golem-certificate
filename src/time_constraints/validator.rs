use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use super::TimeConstraints;

pub fn validate_time_constraints(parent: &TimeConstraints, child: &TimeConstraints) -> Result<()> {
    if parent.not_before <= child.not_before && child.not_after <= parent.not_after {
        Ok(())
    } else {
        Err(anyhow!("Child cannot extend time constraints"))
    }
}

pub fn validate_timestamp(constraints: &TimeConstraints, now: DateTime<Utc>) -> Result<()> {
    if now > constraints.not_after {
        Err(anyhow!("Child is not valid anymore"))
    } else if now < constraints.not_before {
        Err(anyhow!("Child is not valid yet"))
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
        let parent = &TimeConstraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &TimeConstraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };

        assert!(validate_time_constraints(parent, child).is_ok());
    }

    #[test]
    fn accept_because_child_time_constraints_are_the_same_as_parent() {
        let parent = &TimeConstraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &TimeConstraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };

        assert!(validate_time_constraints(parent, child).is_ok());
    }

    #[test]
    fn reject_because_child_has_earlier_not_before() {
        let parent = &TimeConstraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &TimeConstraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };

        assert!(validate_time_constraints(parent, child).is_err());
    }

    #[test]
    fn reject_because_child_has_later_not_after() {
        let parent = &TimeConstraints {
            not_before: dt("2000-01-01T00:00:00Z"),
            not_after: dt("2000-01-01T04:04:04Z"),
        };
        let child = &TimeConstraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T05:05:05Z"),
        };

        assert!(validate_time_constraints(parent, child).is_err());
    }

    #[test]
    fn reject_timestamp_because_constraint_has_expired() {
        let constraints = &TimeConstraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T04:04:04Z");

        assert!(validate_timestamp(constraints, now).is_err());
    }

    #[test]
    fn reject_timestamp_because_constraint_is_not_valid_yet() {
        let constraints = &TimeConstraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T00:00:00Z");

        assert!(validate_timestamp(constraints, now).is_err());
    }

    #[test]
    fn accept_timestamp_because_it_fits_constraints() {
        let constraints = &TimeConstraints {
            not_before: dt("2000-01-01T01:01:01Z"),
            not_after: dt("2000-01-01T03:03:03Z"),
        };
        let now = dt("2000-01-01T02:02:02Z");

        assert!(validate_timestamp(constraints, now).is_ok());
    }

    mod utils {
        use super::*;

        pub fn dt(s: &str) -> DateTime<Utc> {
            s.parse().unwrap()
        }
    }
}
