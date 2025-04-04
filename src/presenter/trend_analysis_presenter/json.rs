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
        Ok(TrendAnalysisResponse::Json {
            company: output.company,
            display_macos_pattern: output.display_macos_pattern,
            rate_of_chance: output.rate_of_chance,
            latest_chance: output.latest_chance,
            ecp1s: output.ecp1s,
            trend_reversal_analysis: output.trend_reversal_analysis,
            indicator_analysis: output.indicator_analysis,
        })
    }
}
