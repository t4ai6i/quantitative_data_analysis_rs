use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct FinancialIndicator {
    pub code: String,
    pub market: String,
    pub target_date: NaiveDate,
}
