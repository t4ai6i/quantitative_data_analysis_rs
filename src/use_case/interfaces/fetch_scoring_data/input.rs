use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FetchScoringData {
    pub code: String,
    pub target_date: NaiveDate,
    pub fetched_at: DateTime<Utc>,
}

impl FetchScoringData {
    pub fn new(code: String, target_date: NaiveDate, fetched_at: DateTime<Utc>) -> Self {
        Self {
            code,
            target_date,
            fetched_at,
        }
    }
}
