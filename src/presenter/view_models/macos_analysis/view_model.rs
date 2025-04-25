use crate::domain::models::macos::model;
use crate::domain::models::macos::model::{Pattern, MACOS};
use crate::domain::models::macos_analysis::close::model::{LatestChance, RateOfChance};
use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::shared::custom_date_format;
use charts_rs::NIL_VALUE;
use chrono::NaiveDate;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOSAnalysis {
    pub pattern: Pattern,
    #[serde(with = "custom_date_format")]
    pub latest_chance: NaiveDate,
    pub rate_of_chance: f64,
}

impl From<(MACOSPatternFilter, RateOfChance, LatestChance)> for MACOSAnalysis {
    fn from(value: (MACOSPatternFilter, RateOfChance, LatestChance)) -> Self {
        let (macos_pattern_filter, rate_of_chance, latest_chance) = value;
        let latest_chance = macos_pattern_filter.get_latest_chance(&latest_chance);
        let pattern = Pattern::from(latest_chance);
        let latest_chance = NaiveDate::from(latest_chance);
        let rate_of_chance = macos_pattern_filter.get_rate_of_chance(&rate_of_chance);
        Self {
            pattern,
            latest_chance,
            rate_of_chance,
        }
    }
}

pub trait MACOSes {
    fn sma_25_closes(&self, filter_pattern: Pattern) -> Vec<f32>;
    fn sma_25_volumes(&self, filter_pattern: Pattern) -> Vec<f32>;
}

impl MACOSes for model::MACOSes {
    fn sma_25_closes(&self, filter_pattern: Pattern) -> Vec<f32> {
        self.par_iter()
            .map(|macos| {
                let MACOS {
                    pattern_close_volume,
                    sma_set_25,
                    ..
                } = macos;
                pattern_close_volume.close.value_if_patterns_match(
                    filter_pattern,
                    sma_set_25.map(|sma_set| sma_set.close),
                )
            })
            .collect()
    }

    fn sma_25_volumes(&self, filter_pattern: Pattern) -> Vec<f32> {
        self.par_iter()
            .map(|macos| {
                let MACOS {
                    pattern_close_volume,
                    sma_set_25,
                    ..
                } = macos;
                pattern_close_volume.volume.value_if_patterns_match(
                    filter_pattern,
                    sma_set_25.map(|sma_set| sma_set.volume),
                )
            })
            .collect()
    }
}

impl Pattern {
    fn value_if_patterns_match(&self, rhs: Self, value: Option<f64>) -> f32 {
        if self.eq(&rhs) {
            value.map_or(NIL_VALUE, |value| value as _)
        } else {
            NIL_VALUE
        }
    }
}
