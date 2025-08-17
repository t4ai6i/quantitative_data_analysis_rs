use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysis {
    pub code: String,
    pub market: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub crossover_pattern_filter: CrossoverPatternFilter,
}

impl TrendAnalysis {
    pub fn new(
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Self {
        Self {
            code: code.into(),
            market: market.into(),
            start_date,
            end_date,
            crossover_pattern_filter,
        }
    }
}
