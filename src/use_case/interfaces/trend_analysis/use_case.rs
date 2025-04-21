use crate::presenter::trend_analysis_presenter::TrendAnalysisOutput;
use crate::use_case::interfaces::trend_analysis::input;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TrendAnalysis {
    async fn handle<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_MIN_RATE: usize,
    >(
        &self,
        input: input::TrendAnalysis,
    ) -> Result<TrendAnalysisOutput<AFTER_DAYS, MARUBOZU_MIN_RATE>>;
}
