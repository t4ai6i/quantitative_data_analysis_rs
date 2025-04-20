use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::trend_summary_presenter::TrendSummaryOutput;
use crate::use_case::interface::trend_summary::input;
use crate::use_case::interface::trend_summary::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendSummary;

#[async_trait]
impl use_case::TrendSummary for TrendSummary {
    async fn handle(&self, input: input::TrendSummary) -> Result<TrendSummaryOutput> {
        Ok(TrendSummaryOutput::new(input.vec_trend_analysis_response))
    }
}
