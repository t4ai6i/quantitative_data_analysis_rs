use crate::domain::entity::cross::CrossDirectionType;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct CrossAnalysis {
    pub(crate) code: String,
    pub(crate) symbol: String,
    pub(crate) cross_direction: CrossDirectionType,
    #[serde(with = "custom_date_format")]
    pub(crate) latest_chance: NaiveDate,
    pub(crate) chance_rate: f64,
}
