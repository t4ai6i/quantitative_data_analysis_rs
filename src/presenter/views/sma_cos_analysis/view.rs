use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::crossover_strategy::latest_chance::model::LatestChance;
use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
use crate::domain::models::crossover_strategy::sma_cos::model;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::shared::custom_serde::naive_date;
use charts_rs::NIL_VALUE;
use chrono::NaiveDate;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SmaCosAnalysis {
    pub crossover_pattern: CrossoverPattern,
    #[serde(with = "naive_date::primitive")]
    pub latest_chance: NaiveDate,
    pub rate_of_chance: f64,
}

impl From<(&CrossoverPatternFilter, &RateOfChance, &LatestChance)> for SmaCosAnalysis {
    fn from(value: (&CrossoverPatternFilter, &RateOfChance, &LatestChance)) -> Self {
        let (crossover_pattern_filter, rate_of_chance, latest_chance) = value;
        let latest_chance = crossover_pattern_filter.get_latest_chance(latest_chance);
        let crossover_pattern = CrossoverPattern::from(latest_chance);
        let latest_chance = NaiveDate::from(latest_chance);
        let rate_of_chance = crossover_pattern_filter.get_rate_of_chance(rate_of_chance);
        Self {
            crossover_pattern,
            latest_chance,
            rate_of_chance,
        }
    }
}

pub trait SmaCoses {
    fn sma_25_closes(&self, filter_pattern: CrossoverPattern) -> Vec<f32>;
    fn sma_25_volumes(&self, filter_pattern: CrossoverPattern) -> Vec<f32>;
}

impl SmaCoses for model::SmaCoses {
    fn sma_25_closes(&self, filter_pattern: CrossoverPattern) -> Vec<f32> {
        collect_filtered_sma_values(self, filter_pattern, |sma_cos| {
            (
                sma_cos.crossover_pattern_close.0,
                sma_cos.close_25.map(|c| c.0),
            )
        })
    }

    fn sma_25_volumes(&self, filter_pattern: CrossoverPattern) -> Vec<f32> {
        collect_filtered_sma_values(self, filter_pattern, |sma_cos| {
            (
                sma_cos.crossover_pattern_volume.0,
                sma_cos.volume_25.map(|v| v.0),
            )
        })
    }
}

fn collect_filtered_sma_values(
    sma_coses: &model::SmaCoses,
    filter_pattern: CrossoverPattern,
    extract: impl Fn(&model::SmaCos) -> (CrossoverPattern, Option<f64>) + Sync,
) -> Vec<f32> {
    sma_coses
        .par_iter()
        .map(|sma_cos| {
            let (crossover_pattern, value) = extract(sma_cos);
            value_if_patterns_match(&crossover_pattern, &filter_pattern, value)
        })
        .collect()
}

fn value_if_patterns_match(
    lhs: &CrossoverPattern,
    rhs: &CrossoverPattern,
    value: Option<f64>,
) -> f32 {
    if lhs.eq(rhs) {
        value.map_or(NIL_VALUE, |value| value as _)
    } else {
        NIL_VALUE
    }
}
