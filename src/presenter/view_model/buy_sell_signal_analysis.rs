use crate::domain::entity::price_action::{BuySellSignalByPriceAction, BuySellSignalType};
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignal {
    pub buy_sell_signal: BuySellSignalType,
    #[serde(with = "custom_date_format")]
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignalAnalysis {
    pub code: String,
    pub symbol: String,
    pub buy_sell_signals: Vec<BuySellSignal>,
}

impl From<&[BuySellSignalByPriceAction]> for BuySellSignalAnalysis {
    fn from(value: &[BuySellSignalByPriceAction]) -> Self {
        let buy_sell_signals = value
            .iter()
            .map(|e| BuySellSignal {
                buy_sell_signal: e.buy_sell_signal,
                date: e.date,
            })
            .collect_vec();
        Self {
            code: "".to_string(),
            symbol: "".to_string(),
            buy_sell_signals,
        }
    }
}
