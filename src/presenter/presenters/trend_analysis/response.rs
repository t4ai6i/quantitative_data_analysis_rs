use crate::domain::models::company::model::Company;
use crate::domain::models::crossover_strategy::latest_chance::model::LatestChance;
use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
use crate::domain::models::high_low_direction_signal::model::HighLowDirectionSignals;
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use deref_derive::{Deref, DerefMut};

pub mod chart;
pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysis {
    Chart {
        company: Company,
        crossover_pattern_filter: CrossoverPatternFilter,
        rate_of_chance: RateOfChance,
        latest_chance: LatestChance,
        body: String,
    },
    Json {
        company: Company,
        crossover_pattern_filter: CrossoverPatternFilter,
        rate_of_chance: RateOfChance,
        latest_chance: LatestChance,
        high_low_direction_signals: HighLowDirectionSignals,
        trend_reversal_analysis: TrendReversalAnalysis,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct TrendAnalyses(pub Vec<TrendAnalysis>);
