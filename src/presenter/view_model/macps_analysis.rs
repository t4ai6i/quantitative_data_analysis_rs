use crate::domain::entity::buy_sell_signal::BuySellSignalType;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACPSAnalysis {
    pub r#type: BuySellSignalType,
    #[serde(with = "custom_date_format")]
    pub date: NaiveDate,
}
