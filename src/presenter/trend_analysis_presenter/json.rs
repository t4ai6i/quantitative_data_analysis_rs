use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
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
        let MACOSAnalysisCloses {
            rate_of_chance,
            latest_chance,
            ..
        } = output.macos_analysis_closes;
        Ok(TrendAnalysisResponse::Json {
            company: output.company,
            display_macos_pattern: output.display_macos_pattern,
            rate_of_chance,
            latest_chance,
            vec_ecp1: output.vec_ecp1,
            trend_reversal_analysis: output.trend_reversal_analysis,
            indicator_analysis: output.indicator_analysis,
        })
    }
}
