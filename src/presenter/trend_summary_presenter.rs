pub mod chart;
pub mod json;

use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use crate::presenter::view_model::analysis::Analysis;
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;
use anyhow::Result;

pub struct TrendSummaryOutput {
    vec_trend_analysis_response: VecTrendAnalysisResponse,
}

impl TrendSummaryOutput {
    pub fn new(vec_trend_analysis_response: VecTrendAnalysisResponse) -> Self {
        Self {
            vec_trend_analysis_response,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendSummaryResponse {
    Chart { body: String },
    JSON { data: Vec<Analysis> },
}

pub trait TrendSummaryPresenter {
    fn handle(
        &self,
        output: TrendSummaryOutput,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendSummaryResponse>;
}
