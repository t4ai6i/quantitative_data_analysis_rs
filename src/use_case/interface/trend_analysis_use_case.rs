use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct TrendAnalysisInput {
    pub code: String,
    pub market: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub display_macos_pattern: DisplayMACOSPattern,
}

impl TrendAnalysisInput {
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

#[async_trait]
pub trait TrendAnalysisUseCase {
    async fn handle<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_MIN_RATE: usize,
    >(
        &self,
        input: TrendAnalysisInput,
    ) -> Result<TrendAnalysisOutput<AFTER_DAYS, MARUBOZU_MIN_RATE>>;
}
