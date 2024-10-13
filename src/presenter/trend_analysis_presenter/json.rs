use crate::domain::entity::candle_stick_pattern_analysis::CandleStickPatternAnalysis;
use crate::domain::entity::close_macos_trend_analysis::VecCloseMACOSTrendAnalysis;
use crate::domain::entity::macos::MACOSType;
use crate::domain::entity::macos::MACOSType::{Dead, Golden};
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
        let latest_golden_macos = Self::get_latest_macos(&Golden, &output);
        let latest_dead_macos = Self::get_latest_macos(&Dead, &output);
        let CandleStickPatternAnalysis {
            latest_ecp2_buy,
            latest_ecp2_sell,
            latest_msesp_buy,
            latest_msesp_sell,
        } = output.candle_stick_pattern_analysis;
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
            latest_golden_macos,
            latest_dead_macos,
            latest_ecp2_buy,
            latest_ecp2_sell,
            latest_msesp_buy,
            latest_msesp_sell,
            vec_ecp1: output.vec_ecp1,
            macps: output.macps,
        })
    }
}

impl JSON {
    fn get_latest_macos<const N: usize, const M: usize>(
        r#type: &MACOSType,
        output: &TrendAnalysisOutput<{ N }, { M }>,
    ) -> Option<NaiveDate> {
        output
            .vec_macos
            .0
            .as_slice()
            .iter()
            .filter(|macos| macos.macos_set_5_25.close.eq(r#type))
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .rev()
            .last()
            .map(|x| x.date)
    }
}
