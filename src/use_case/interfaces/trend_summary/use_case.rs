use crate::presenter::trend_summary_presenter::TrendSummaryOutput;
use crate::use_case::interfaces::trend_summary::input;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TrendSummary {
    async fn handle(&self, input: input::TrendSummary) -> Result<TrendSummaryOutput>;
}
