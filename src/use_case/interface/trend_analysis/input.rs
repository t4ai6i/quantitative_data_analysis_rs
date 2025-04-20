use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysis {
    pub code: String,
    pub market: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub display_macos_pattern: DisplayMACOSPattern,
}

impl TrendAnalysis {
    pub fn new(
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        display_macos_pattern: DisplayMACOSPattern,
    ) -> Self {
        Self {
            code: code.into(),
            market: market.into(),
            start_date,
            end_date,
            display_macos_pattern,
        }
    }
}
