use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use super::ValidityPeriods;

pub fn validate_validity_periods(parent: &ValidityPeriods, child: &ValidityPeriods) -> Result<()> {
    if parent.not_before <= child.not_before && child.not_after <= parent.not_after {
        Ok(())
    } else {
        Err(anyhow!(
            "Child cannot extend time periods, parent: {:?}, child: {:?}",
            parent,
            child
        ))
    }
}

pub fn validate_timestamp(periods: &ValidityPeriods, ts: DateTime<Utc>) -> Result<()> {
    if periods.not_before <= ts && ts <= periods.not_after {
        Ok(())
    } else {
        Err(anyhow!(
            "Timestamp: {ts} is not between given periods: {:?}",
            periods
        ))
    }
}

#[cfg(test)]
mod tests {
    use utils::*;

    use super::*;

    mod validate_validity_periods_should {
        use super::*;

        #[test]
        pub(crate) fn accept_because_child_periods_are_subset_of_parent() {
            let parent = &ValidityPeriods {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };

            assert!(validate_validity_periods(parent, child).is_ok());
        }

        #[test]
        pub(crate) fn accept_because_child_periods_are_the_same_as_parent() {
            let parent = &ValidityPeriods {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriods {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };

            assert!(validate_validity_periods(parent, child).is_ok());
        }

        #[test]
        pub(crate) fn reject_because_child_has_earlier_not_before() {
            let parent = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriods {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };

            assert!(validate_validity_periods(parent, child).is_err());
        }

        #[test]
        pub(crate) fn reject_because_child_has_later_not_after() {
            let parent = &ValidityPeriods {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T05:05:05Z"),
            };

            assert!(validate_validity_periods(parent, child).is_err());
        }
    }

    mod validate_timestamp_should {
        use super::*;

        #[test]
        pub(crate) fn reject_timestamp_because_period_has_expired() {
            let periods = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T04:04:04Z");

            assert!(validate_timestamp(periods, now).is_err());
        }

        #[test]
        pub(crate) fn reject_timestamp_because_period_is_not_valid_yet() {
            let periods = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T00:00:00Z");

            assert!(validate_timestamp(periods, now).is_err());
        }

        #[test]
        pub(crate) fn accept_timestamp_because_it_fits_periods() {
            let periods = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T02:02:02Z");

            assert!(validate_timestamp(periods, now).is_ok());
        }

        #[test]
        pub(crate) fn accept_timestamp_because_it_is_the_same_as_not_before() {
            let periods = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T01:01:01Z");

            assert!(validate_timestamp(periods, now).is_ok());
        }

        #[test]
        pub(crate) fn accept_timestamp_because_it_is_the_same_as_not_after() {
            let periods = &ValidityPeriods {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T03:03:03Z");

            assert!(validate_timestamp(periods, now).is_ok());
        }
    }

    mod utils {
        use super::*;

        pub fn dt(s: &str) -> DateTime<Utc> {
            s.parse().unwrap()
        }
    }
}
