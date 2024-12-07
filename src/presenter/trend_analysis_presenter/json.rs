use crate::domain::entity::close_macos_trend_analysis::VecCloseMACOSTrendAnalysis;
use crate::presenter::trend_analysis_presenter::{
    TrendAnalysisOutput, TrendAnalysisPresenter, TrendAnalysisResponse,
};
use anyhow::Result;

pub struct JSON;

impl TrendAnalysisPresenter for JSON {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse> {
        let VecCloseMACOSTrendAnalysis {
            chance_rate,
            latest_chance,
            ..
        } = output.vec_close_macos_trend_analysis;
        Ok(TrendAnalysisResponse::Json {
            company: output.company,
            display_macos_pattern: output.display_macos_pattern,
            chance_rate,
            latest_chance,
            vec_ecp1: output.vec_ecp1,
            trend_reversal_analysis: output.trend_reversal_analysis,
            indicator_analysis: output.indicator_analysis,
        })
    }
}
