use crate::domain::entity::close_macos_trend_analysis::{ChanceRate, LatestChance};
use crate::domain::entity::macos::MACOSType;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOSAnalysis {
    pub r#type: MACOSType,
    #[serde(with = "custom_date_format")]
    pub latest_chance: NaiveDate,
    pub chance_rate: f64,
}

impl From<(DisplayMACOSPattern, ChanceRate, LatestChance)> for MACOSAnalysis {
    fn from(value: (DisplayMACOSPattern, ChanceRate, LatestChance)) -> Self {
        let (display_macos_pattern, chance_rate, latest_chance) = value;
        let latest_chance = display_macos_pattern.get_latest_chance(&latest_chance);
        let r#type = MACOSType::from(latest_chance);
        let latest_chance = NaiveDate::from(latest_chance);
        let chance_rate = display_macos_pattern.get_chance_rate(&chance_rate);
        Self {
            r#type,
            latest_chance,
            chance_rate,
        }
    }
}
