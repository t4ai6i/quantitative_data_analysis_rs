use crate::domain::entity::indicator_analysis::IndicatorAnalysis;
use crate::domain::entity::sma::VecSMA;
use crate::domain::entity::trend_reversal_analysis::TrendReversalAnalysis;
use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::company::model::Company;
use crate::domain::models::ecp1::model::ECP1s;
use crate::domain::models::macos::model::MACOSES;
use crate::domain::models::macos_analysis::close::model::{
    LatestChance, MACOSAnalysisCloses, RateOfChance,
};
use crate::domain::models::macos_analysis::volume::model::MACOSAnalysisVolumes;
use crate::domain::models::stock::model::Stocks;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use anyhow::Result;

pub mod chart;
pub mod json;

pub struct TrendAnalysisOutput<const N: usize, const M: usize> {
    pub company: Company,
    pub stocks: Stocks,
    pub vec_sma_5: VecSMA<5>,
    pub vec_sma_25: VecSMA<25>,
    pub vec_sma_50: VecSMA<50>,
    pub macoses: MACOSES,
    pub macos_analysis_closes: MACOSAnalysisCloses<N>,
    pub rate_of_chance: RateOfChance,
    pub latest_chance: LatestChance,
    pub macos_analysis_volumes: MACOSAnalysisVolumes,
    pub ecp1s: ECP1s,
    pub candle_sticks: CandleSticks<M>,
    pub trend_reversal_analysis: TrendReversalAnalysis,
    pub indicator_analysis: IndicatorAnalysis,
    pub display_macos_pattern: DisplayMACOSPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysisResponse {
    Chart {
        company: Company,
        display_macos_pattern: DisplayMACOSPattern,
        rate_of_chance: RateOfChance,
        latest_chance: LatestChance,
        body: String,
    },
    Json {
        company: Company,
        display_macos_pattern: DisplayMACOSPattern,
        rate_of_chance: RateOfChance,
        latest_chance: LatestChance,
        ecp1s: ECP1s,
        trend_reversal_analysis: TrendReversalAnalysis,
        indicator_analysis: IndicatorAnalysis,
    },
}

pub trait TrendAnalysisPresenter {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse>;
}
