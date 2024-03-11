use crate::domain::entity::ordering::Ordering;
use serde::{Deserialize, Serialize};
use std::cmp;
use strum::Display;

/// 売買シグナル
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum BuySellSignalType {
    #[default]
    #[serde(rename = "wait_and_see")]
    WaitAndSee,
    #[serde(rename = "buy_signal")]
    BuySignal,
    #[serde(rename = "sell_signal")]
    SellSignal,
}

impl From<(Ordering<1, 1>, Ordering<1, 1>)> for BuySellSignalType {
    fn from(value: (Ordering<1, 1>, Ordering<1, 1>)) -> Self {
        let (low, high) = value;
        let low = low.0;
        let high = high.0;
        match (low, high) {
            (Some(cmp::Ordering::Greater), Some(cmp::Ordering::Greater)) => Self::BuySignal,
            (Some(cmp::Ordering::Less), Some(cmp::Ordering::Less)) => Self::SellSignal,
            _ => Self::WaitAndSee,
        }
    }
}
