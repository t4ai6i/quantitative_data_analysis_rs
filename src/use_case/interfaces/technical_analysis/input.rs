use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TechnicalAnalysis {
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub analysis_at: DateTime<Utc>,
}

impl TechnicalAnalysis {
    pub fn new(
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        analysis_at: DateTime<Utc>,
    ) -> Self {
        Self {
            code: code.into(),
            start_date,
            end_date,
            analysis_at,
        }
    }
}
