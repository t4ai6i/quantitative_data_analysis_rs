use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::shared::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct BuySellSignal {
    pub r#type: BuySellSignalType,
    #[serde(with = "custom_date_format")]
    pub date: NaiveDate,
}
