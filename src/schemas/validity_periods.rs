use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod validator;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ValidityPeriod {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}
