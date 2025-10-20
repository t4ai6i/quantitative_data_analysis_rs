use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::shared::custom_date_format;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACPSAnalysis {
    pub r#type: BuySellSignalType,
    #[serde(with = "custom_date_format::primitive")]
    pub date: NaiveDate,
}
