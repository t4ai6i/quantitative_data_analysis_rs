use deref_derive::{Deref, DerefMut};

use crate::domain::models::company::model::Company;
use crate::domain::models::ecp1::model::ECP1s;
use crate::domain::models::indicator_analysis::model::IndicatorAnalysis;
use crate::domain::models::macos_analysis::close::model::{LatestChance, RateOfChance};
use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
use crate::presenter::macos_pattern_filter::MACOSPatternFilter;

pub mod chart;
pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysis {
    Chart {
        company: Company,
        macos_pattern_filter: MACOSPatternFilter,
        rate_of_chance: RateOfChance,
        latest_chance: LatestChance,
        body: String,
    },
    Json {
        company: Company,
        macos_pattern_filter: MACOSPatternFilter,
        rate_of_chance: RateOfChance,
        latest_chance: LatestChance,
        ecp1s: ECP1s,
        trend_reversal_analysis: TrendReversalAnalysis,
        indicator_analysis: IndicatorAnalysis,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct TrendAnalyses(pub Vec<TrendAnalysis>);
