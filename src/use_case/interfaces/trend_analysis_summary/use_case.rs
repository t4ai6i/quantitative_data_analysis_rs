use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::trend_analysis_summary::output;
use crate::use_case::interfaces::trend_analysis_summary::input;

#[async_trait]
pub trait TrendAnalysisSummary {
    async fn handle(
        &self,
        input: input::TrendAnalysisSummary,
    ) -> Result<output::TrendAnalysisSummary>;
}
