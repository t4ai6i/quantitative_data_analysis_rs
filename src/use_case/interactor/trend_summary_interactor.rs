use crate::presenter::trend_summary_presenter::TrendSummaryOutput;
use crate::use_case::interface::trend_summary_use_case::{TrendSummaryInput, TrendSummaryUseCase};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendSummaryInteractor;

impl TrendSummaryInteractor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl TrendSummaryUseCase for TrendSummaryInteractor {
    async fn handle(&self, input: TrendSummaryInput) -> Result<TrendSummaryOutput> {
        Ok(TrendSummaryOutput::new(input.vec_trend_analysis_response))
    }
}
