use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::company::model::Company;
use crate::domain::models::ecp1::model::ECP1s;
use crate::domain::models::indicator_analysis::model::IndicatorAnalysis;
use crate::domain::models::macos::model::MACOSes;
use crate::domain::models::macos_analysis::close::model::{
    LatestChance, MACOSAnalysisCloses, RateOfChance,
};
use crate::domain::models::macos_analysis::volume::model::MACOSAnalysisVolumes;
use crate::domain::models::sma::model::SMAs;
use crate::domain::models::stock::model::Stocks;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use anyhow::Result;

pub mod chart;
pub mod json;

pub struct TrendAnalysisOutput<const N: usize, const M: usize> {
    pub company: Company,
    pub stocks: Stocks,
    pub smas_5: SMAs<5>,
    pub smas_25: SMAs<25>,
    pub smas_50: SMAs<50>,
    pub macoses: MACOSes,
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
