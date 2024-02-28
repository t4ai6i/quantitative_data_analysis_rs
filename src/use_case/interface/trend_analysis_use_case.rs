use crate::presenter::trend_analysis_presenter::{DisplayCrossPattern, TrendAnalysisOutput};
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysisInput {
    pub code: String,
    pub market: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub display_cross_pattern: DisplayCrossPattern,
}

impl TrendAnalysisInput {
    pub fn new(
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Self {
        Self {
            code: code.into(),
            market: market.into(),
            start_date,
            end_date,
            display_cross_pattern,
        }
    }
}

#[async_trait]
pub trait TrendAnalysisUseCase {
    async fn handle<const AFTER_DAYS: usize, const FOR_DAYS: usize>(
        &self,
        input: TrendAnalysisInput,
    ) -> Result<TrendAnalysisOutput<AFTER_DAYS>>;
}
