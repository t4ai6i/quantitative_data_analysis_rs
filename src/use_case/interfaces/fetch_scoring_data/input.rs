use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct FetchScoringData {
    pub code: String,
    pub target_date: NaiveDate,
}

impl FetchScoringData {
    pub fn new(code: String, target_date: NaiveDate) -> Self {
        Self { code, target_date }
    }
}
