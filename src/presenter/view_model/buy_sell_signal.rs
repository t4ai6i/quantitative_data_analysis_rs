use crate::domain::entity::buy_sell_signal::BuySellSignalType;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignal {
    pub r#type: BuySellSignalType,
    #[serde(with = "custom_date_format")]
    pub date: NaiveDate,
}
