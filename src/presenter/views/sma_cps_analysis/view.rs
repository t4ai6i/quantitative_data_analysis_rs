use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::shared::custom_serde::naive_date;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SmaCpsAnalysis {
    pub r#type: BuySellSignalType,
    #[serde(with = "naive_date::primitive")]
    pub date: NaiveDate,
}
