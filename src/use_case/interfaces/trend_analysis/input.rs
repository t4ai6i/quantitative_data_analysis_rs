use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysis {
    pub code: String,
    pub market: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub macos_pattern_filter: MACOSPatternFilter,
}

impl TrendAnalysis {
    pub fn new(
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        macos_pattern_filter: MACOSPatternFilter,
    ) -> Self {
        Self {
            code: code.into(),
            market: market.into(),
            start_date,
            end_date,
            macos_pattern_filter,
        }
    }
}
