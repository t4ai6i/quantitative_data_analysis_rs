use crate::domain::entity::candle_stick::VecCandleStick;
use crate::domain::entity::candle_stick_pattern_analysis::CandleStickPatternAnalysis;
use crate::domain::entity::close_cross_trend_analysis::{
    ChanceRate, LatestChance, VecCloseCrossTrendAnalysis,
};
use crate::domain::entity::company::Company;
use crate::domain::entity::cross::VecCross;
use crate::domain::entity::ecp1::VecECP1;
use crate::domain::entity::sma::VecSMA;
use crate::domain::entity::stock::VecStock;
use crate::domain::entity::volume_cross_trend_analysis::VecVolumeCrossTrendAnalysis;
use crate::presenter::display_cross_pattern::DisplayCrossPattern;
use anyhow::Result;
use chrono::NaiveDate;

pub mod chart;
pub mod json;

pub struct TrendAnalysisOutput<const N: usize, const M: usize> {
    pub company: Company,
    pub vec_stock: VecStock,
    pub vec_sma_5: VecSMA<5>,
    pub vec_sma_25: VecSMA<25>,
    pub vec_cross: VecCross,
    pub vec_close_cross_trend_analysis: VecCloseCrossTrendAnalysis<N>,
    pub vec_volume_cross_trend_analysis: VecVolumeCrossTrendAnalysis,
    pub vec_ecp1: VecECP1,
    pub vec_candle_stick: VecCandleStick<M>,
    pub candle_stick_pattern_analysis: CandleStickPatternAnalysis,
    pub display_cross_pattern: DisplayCrossPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysisResponse {
    Chart {
        company: Company,
        display_cross_pattern: DisplayCrossPattern,
        chance_rate: ChanceRate,
        latest_chance: LatestChance,
        body: String,
    },
    Json {
        company: Company,
        display_cross_pattern: DisplayCrossPattern,
        chance_rate: ChanceRate,
        latest_chance: LatestChance,
        latest_golden_cross: Option<NaiveDate>,
        latest_dead_cross: Option<NaiveDate>,
        latest_ecp2_buy: Option<NaiveDate>,
        latest_ecp2_sell: Option<NaiveDate>,
        latest_msesp_buy: Option<NaiveDate>,
        latest_msesp_sell: Option<NaiveDate>,
        vec_ecp1: VecECP1,
    },
}

pub trait TrendAnalysisPresenter {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse>;
}
