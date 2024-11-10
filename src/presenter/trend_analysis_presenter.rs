use crate::domain::entity::candle_stick::VecCandleStick;
use crate::domain::entity::close_macos_trend_analysis::{
    ChanceRate, LatestChance, VecCloseMACOSTrendAnalysis,
};
use crate::domain::entity::company::Company;
use crate::domain::entity::ecp1::VecECP1;
use crate::domain::entity::macos::VecMACOS;
use crate::domain::entity::sma::VecSMA;
use crate::domain::entity::stock::VecStock;
use crate::domain::entity::trend_reversal_analysis::TrendReversalAnalysis;
use crate::domain::entity::volume_macos_trend_analysis::VecVolumeMACOSTrendAnalysis;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use anyhow::Result;

pub mod chart;
pub mod json;

pub struct TrendAnalysisOutput<const N: usize, const M: usize> {
    pub company: Company,
    pub vec_stock: VecStock,
    pub vec_sma_5: VecSMA<5>,
    pub vec_sma_25: VecSMA<25>,
    pub vec_sma_50: VecSMA<50>,
    pub vec_macos: VecMACOS,
    pub vec_close_macos_trend_analysis: VecCloseMACOSTrendAnalysis<N>,
    pub vec_volume_macos_trend_analysis: VecVolumeMACOSTrendAnalysis,
    pub vec_ecp1: VecECP1,
    pub vec_candle_stick: VecCandleStick<M>,
    pub trend_reversal_analysis: TrendReversalAnalysis,
    pub display_macos_pattern: DisplayMACOSPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysisResponse {
    Chart {
        company: Company,
        display_macos_pattern: DisplayMACOSPattern,
        chance_rate: ChanceRate,
        latest_chance: LatestChance,
        body: String,
    },
    Json {
        company: Company,
        display_macos_pattern: DisplayMACOSPattern,
        chance_rate: ChanceRate,
        latest_chance: LatestChance,
        vec_ecp1: VecECP1,
        trend_reversal_analysis: TrendReversalAnalysis,
    },
}

pub trait TrendAnalysisPresenter {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse>;
}
