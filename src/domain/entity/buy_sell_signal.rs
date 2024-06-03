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

pub type CompareLowForTwoDays = Ordering<1, 1>;
pub type CompareHighForTwoDays = Ordering<1, 1>;
/// Engulfing Candlestick Pattern1
///
/// ある日とその前日の安値・高値の切り上がり・切り下がりをみる
///
/// * **BuySignal**: 安値切り上がりかつ高値切り上がり
/// * **SellSignal**: 安値切り下がりかつ高値切り下がり
/// * **WaitAndSee**: 上記のどちらにもあてはまならない
///
pub type ECP1 = (CompareLowForTwoDays, CompareHighForTwoDays);

impl From<ECP1> for BuySellSignalType {
    fn from(value: ECP1) -> Self {
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
