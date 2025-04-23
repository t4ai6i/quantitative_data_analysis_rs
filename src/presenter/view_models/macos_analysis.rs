use crate::domain::models::macos::model::Pattern;
use crate::domain::models::macos_analysis::close::model::{LatestChance, RateOfChance};
use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::shared::custom_date_format;
use chrono::NaiveDate;
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
