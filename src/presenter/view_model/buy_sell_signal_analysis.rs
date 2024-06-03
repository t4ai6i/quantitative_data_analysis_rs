use crate::domain::entity::buy_sell_signal::BuySellSignalType;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignal {
    pub buy_sell_signal: BuySellSignalType,
    #[serde(with = "custom_date_format")]
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignalAnalysis {
    pub buy_sell_signals: Vec<BuySellSignal>,
}
