use chrono::{DateTime, Utc};

use super::ValidityPeriod;
use crate::Error;

pub fn validate_validity_period(
    parent: &ValidityPeriod,
    child: &ValidityPeriod,
) -> Result<(), Error> {
    if parent.not_before <= child.not_before && child.not_after <= parent.not_after {
        Ok(())
    } else {
        Err(Error::ValidityPeriodExtended {
            parent: parent.to_owned(),
            child: child.to_owned(),
        })
    }
}

pub fn validate_timestamp(period: &ValidityPeriod, ts: DateTime<Utc>) -> Result<(), Error> {
    if period.not_before > ts {
        Err(Error::NotValidYet(period.not_before))
    } else if ts > period.not_after {
        Err(Error::Expired(period.not_after))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use utils::*;

    use super::*;

    mod validate_validity_period_should {
        use super::*;

        #[test]
        pub(crate) fn accept_because_child_period_are_subset_of_parent() {
            let parent = &ValidityPeriod {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };

            assert!(validate_validity_period(parent, child).is_ok());
        }

        #[test]
        pub(crate) fn accept_because_child_period_are_the_same_as_parent() {
            let parent = &ValidityPeriod {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriod {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };

            assert!(validate_validity_period(parent, child).is_ok());
        }

        #[test]
        pub(crate) fn reject_because_child_has_earlier_not_before() {
            let parent = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriod {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };

            assert!(validate_validity_period(parent, child).is_err());
        }

        #[test]
        pub(crate) fn reject_because_child_has_later_not_after() {
            let parent = &ValidityPeriod {
                not_before: dt("2000-01-01T00:00:00Z"),
                not_after: dt("2000-01-01T04:04:04Z"),
            };
            let child = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T05:05:05Z"),
            };

            assert!(validate_validity_period(parent, child).is_err());
        }
    }

    mod validate_timestamp_should {
        use super::*;

        #[test]
        pub(crate) fn reject_timestamp_because_period_has_expired() {
            let period = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T04:04:04Z");

            assert!(validate_timestamp(period, now).is_err());
        }

        #[test]
        pub(crate) fn reject_timestamp_because_period_is_not_valid_yet() {
            let period = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T00:00:00Z");

            assert!(validate_timestamp(period, now).is_err());
        }

        #[test]
        pub(crate) fn accept_timestamp_because_it_fits_period() {
            let period = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T02:02:02Z");

            assert!(validate_timestamp(period, now).is_ok());
        }

        #[test]
        pub(crate) fn accept_timestamp_because_it_is_the_same_as_not_before() {
            let period = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T01:01:01Z");

            assert!(validate_timestamp(period, now).is_ok());
        }

        #[test]
        pub(crate) fn accept_timestamp_because_it_is_the_same_as_not_after() {
            let period = &ValidityPeriod {
                not_before: dt("2000-01-01T01:01:01Z"),
                not_after: dt("2000-01-01T03:03:03Z"),
            };
            let now = dt("2000-01-01T03:03:03Z");

            assert!(validate_timestamp(period, now).is_ok());
        }
    }

    mod utils {
        use super::*;

        pub fn dt(s: &str) -> DateTime<Utc> {
            s.parse().unwrap()
        }
    }
}
