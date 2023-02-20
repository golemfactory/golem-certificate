use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub mod validator;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TimeConstraints {
    pub not_before: DateTime<Utc>,
    pub not_after: DateTime<Utc>,
}
