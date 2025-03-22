use crate::domain::entity::close_macos_trend_analysis::{LatestChance, MACOSRateOfChance};
use crate::domain::models::macos::model::Pattern;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOSAnalysis {
    pub pattern: Pattern,
    #[serde(with = "custom_date_format")]
    pub latest_chance: NaiveDate,
    pub rate_of_chance: f64,
}

impl From<(DisplayMACOSPattern, MACOSRateOfChance, LatestChance)> for MACOSAnalysis {
    fn from(value: (DisplayMACOSPattern, MACOSRateOfChance, LatestChance)) -> Self {
        let (display_macos_pattern, rate_of_chance, latest_chance) = value;
        let latest_chance = display_macos_pattern.get_latest_chance(&latest_chance);
        let pattern = Pattern::from(latest_chance);
        let latest_chance = NaiveDate::from(latest_chance);
        let rate_of_chance = display_macos_pattern.get_rate_of_chance(&rate_of_chance);
        Self {
            pattern,
            latest_chance,
            rate_of_chance,
        }
    }
}
