use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::company::model::Company;
use crate::domain::models::crossover_strategy::latest_chance::model::LatestChance;
use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
use crate::domain::models::crossover_strategy::sma_cos::analysis_result;
use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
use crate::domain::models::high_low_direction_signal::model::HighLowDirectionSignals;
use crate::domain::models::sma::model::SMAs;
use crate::domain::models::stock::model::Stocks;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;

pub struct TrendAnalysis<const N: usize, const M: usize, const O: usize, const P: usize> {
    pub company: Company,
    pub stocks: Stocks,
    pub smas_5: SMAs<5>,
    pub smas_25: SMAs<25>,
    pub smas_50: SMAs<50>,
    pub sma_coses: SmaCoses,
    pub sma_cos_analysis_result_closes: analysis_result::close::model::AnalysisResults<N>,
    pub rate_of_chance: RateOfChance,
    pub latest_chance: LatestChance,
    pub sma_cos_analysis_result_volumes: analysis_result::volume::model::AnalysisResults,
    pub high_low_direction_signals: HighLowDirectionSignals,
    pub candle_sticks: CandleSticks<M, O, P>,
    pub trend_reversal_analysis: TrendReversalAnalysis,
    pub crossover_pattern_filter: CrossoverPatternFilter,
}
