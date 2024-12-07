use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Statement {
    pub code: String,
    pub disclosed_date: NaiveDate,
    /// Earnings Per Share
    pub eps: f64,
    /// Book-value Per Share
    pub bps: f64,
}
