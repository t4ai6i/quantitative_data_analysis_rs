use anyhow::Result;

use crate::presenter::display_cross_pattern::DisplayCrossPattern;
use crate::presenter::trend_summary_presenter::{
    TrendSummaryOutput, TrendSummaryPresenter, TrendSummaryResponse,
};
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponseExt;

pub struct JSON;

impl TrendSummaryPresenter for JSON {
    fn handle(
        &self,
        output: TrendSummaryOutput,
        _display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendSummaryResponse> {
        let data = output.vec_trend_analysis_response.vec_json();
        Ok(TrendSummaryResponse::JSON { data })
    }
}
