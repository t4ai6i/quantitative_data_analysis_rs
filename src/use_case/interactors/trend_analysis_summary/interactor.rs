use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::trend_analysis_summary::output;
use crate::use_case::interfaces::trend_analysis_summary::input;
use crate::use_case::interfaces::trend_analysis_summary::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisSummary;

#[async_trait]
impl use_case::TrendAnalysisSummary for TrendAnalysisSummary {
    async fn handle(
        &self,
        input: input::TrendAnalysisSummary,
    ) -> Result<output::TrendAnalysisSummary> {
        Ok(output::TrendAnalysisSummary::new(
            input.trend_analyses,
            input.vec_trend_analysis,
        ))
    }
}
