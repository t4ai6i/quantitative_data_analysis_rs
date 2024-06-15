use crate::domain::entity::cross::CrossDirectionType;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct CrossAnalysis {
    pub cross_direction: CrossDirectionType,
    #[serde(with = "custom_date_format")]
    pub latest_chance: NaiveDate,
    pub chance_rate: f64,
}
