use crate::presenter::presenters::trend_summary::output;
use crate::use_case::interfaces::trend_summary::input;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TrendSummary {
    async fn handle(&self, input: input::TrendSummary) -> Result<output::TrendSummary>;
}
