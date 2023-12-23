use crate::presenter::trend_summary_presenter::TrendSummaryOutput;
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;
use anyhow::Result;
use async_trait::async_trait;

pub struct TrendSummaryInput {
    pub vec_trend_analysis_response: VecTrendAnalysisResponse,
}

impl TrendSummaryInput {
    pub fn new(vec_trend_analysis_response: VecTrendAnalysisResponse) -> Self {
        Self {
            vec_trend_analysis_response,
        }
    }
}

#[async_trait]
pub trait TrendSummaryUseCase {
    async fn handle(&self, input: TrendSummaryInput) -> Result<TrendSummaryOutput>;
}
