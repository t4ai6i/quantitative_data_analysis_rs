use anyhow::Result;

use crate::presenter::trend_analysis_presenter::{
    TrendAnalysisOutput, TrendAnalysisPresenter, TrendAnalysisResponse,
};

pub struct JSON;

impl JSON {
    pub fn new() -> Self {
        Self
    }
}

impl TrendAnalysisPresenter for JSON {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse> {
        Ok(TrendAnalysisResponse::Json {
            company: output.company,
            display_cross_pattern: output.display_cross_pattern,
            chance_rate: output.vec_trend.chance_rate,
            latest_chance: output.vec_trend.latest_chance,
            vec_ecp1: output.vec_ecp1,
            vec_ecp2: output.vec_ecp2,
        })
    }
}
