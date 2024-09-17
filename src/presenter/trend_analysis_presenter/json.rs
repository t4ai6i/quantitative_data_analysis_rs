use crate::domain::entity::candle_stick_pattern_analysis::CandleStickPatternAnalysis;
use crate::domain::entity::close_cross_trend_analysis::VecCloseCrossTrendAnalysis;
use crate::domain::entity::cross::CrossDirectionType;
use crate::domain::entity::cross::CrossDirectionType::{Dead, Golden};
use crate::presenter::trend_analysis_presenter::{
    TrendAnalysisOutput, TrendAnalysisPresenter, TrendAnalysisResponse,
};
use anyhow::Result;
use chrono::NaiveDate;
use itertools::Itertools;

pub struct JSON;

impl TrendAnalysisPresenter for JSON {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse> {
        let latest_golden_cross = Self::get_latest_cross(&Golden, &output);
        let latest_dead_cross = Self::get_latest_cross(&Dead, &output);
        let CandleStickPatternAnalysis {
            latest_ecp2_buy,
            latest_ecp2_sell,
            latest_msesp_buy,
            latest_msesp_sell,
        } = output.candle_stick_pattern_analysis;
        let VecCloseCrossTrendAnalysis {
            chance_rate,
            latest_chance,
            ..
        } = output.vec_close_cross_trend_analysis;
        Ok(TrendAnalysisResponse::Json {
            company: output.company,
            display_cross_pattern: output.display_cross_pattern,
            chance_rate,
            latest_chance,
            latest_golden_cross,
            latest_dead_cross,
            latest_ecp2_buy,
            latest_ecp2_sell,
            latest_msesp_buy,
            latest_msesp_sell,
            vec_ecp1: output.vec_ecp1,
        })
    }
}

impl JSON {
    fn get_latest_cross<const N: usize, const M: usize>(
        r#type: &CrossDirectionType,
        output: &TrendAnalysisOutput<{ N }, { M }>,
    ) -> Option<NaiveDate> {
        output
            .vec_cross
            .0
            .as_slice()
            .iter()
            .filter(|cross| cross.cross_direction_5_25.close_average.eq(r#type))
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .rev()
            .last()
            .map(|x| x.date)
    }
}
