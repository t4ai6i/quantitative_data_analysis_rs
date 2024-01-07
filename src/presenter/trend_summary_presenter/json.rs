use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use crate::presenter::trend_summary_presenter::{
    TrendSummaryOutput, TrendSummaryPresenter, TrendSummaryResponse,
};
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponseExt;
use anyhow::Result;

pub struct JSON;

impl JSON {
    pub fn new() -> Self {
        Self
    }
}

impl TrendSummaryPresenter for JSON {
    fn handle(
        &self,
        output: TrendSummaryOutput,
        _display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendSummaryResponse> {
        let vec_json = output.vec_trend_analysis_response.vec_json();
        Ok(TrendSummaryResponse::JSON { data: vec_json })
    }
}
