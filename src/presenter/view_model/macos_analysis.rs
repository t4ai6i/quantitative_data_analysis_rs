use crate::domain::entity::macos::MACOSType;
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
