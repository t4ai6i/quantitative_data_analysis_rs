use anyhow::Result;

use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use crate::presenter::view_model::analysis::Analysis;
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;

pub mod chart;
pub mod json;

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
        display_macos_pattern: DisplayMACOSPattern,
    ) -> Result<TrendSummaryResponse>;
}
