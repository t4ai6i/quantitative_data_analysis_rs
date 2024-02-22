use crate::domain::entity::ordering::Ordering;
use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::cmp;
use strum::Display;

/// 売買シグナル
#[derive(
Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum BuySellSignal {
    #[default]
    #[serde(rename = "wait_and_see")]
    WaitAndSee,
    #[serde(rename = "buy_signal")]
    BuySignal,
    #[serde(rename = "sell_signal")]
    SellSignal,
}

/// 高値安値からの売買シグナル指標
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct BuySellSignalByPriceAction {
    /// 前日・当日それぞぞれの高値が切り上がったか切り下がったか
    high_trend: Ordering<1, 1>,
    /// 前日・当日それぞぞれの安値が切り上がったか切り下がったか
    low_trend: Ordering<1, 1>,
    /// 高値・安値の切り上げ・切り下げを基にした売買シグナル
    buy_sell_signal: BuySellSignal,
    /// シグナルの対象の日付
    date: NaiveDate,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct VecBuySellSignalByPriceAction(pub Vec<BuySellSignalByPriceAction>);

impl From<(f64, f64)> for Ordering<1, 1> {
    fn from(value: (f64, f64)) -> Self {
        let (prev, today) = value;
        let ordering = today.partial_cmp(&prev);
        Ordering::<1, 1>(ordering)
    }
}

impl From<(Ordering<1, 1>, Ordering<1, 1>)> for BuySellSignal {
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

impl From<&[Stock]> for VecBuySellSignalByPriceAction {
    fn from(value: &[Stock]) -> Self {
        let vec = value
            .windows(2)
            .map(|stock| {
                let prev = stock.get(0).unwrap();
                let today = stock.get(1).unwrap();
                let prev_high = prev.high;
                let today_high = today.high;
                let prev_low = prev.low;
                let today_low = today.low;
                let low_trend = Ordering::<1, 1>::from((prev_low, today_low));
                let high_trend = Ordering::<1, 1>::from((prev_high, today_high));
                let buy_sell_signal = BuySellSignal::from((high_trend, low_trend));
                let date = today.date;
                BuySellSignalByPriceAction {
                    high_trend,
                    low_trend,
                    buy_sell_signal,
                    date,
                }
            })
            .collect_vec();
        VecBuySellSignalByPriceAction(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::price_action::VecBuySellSignalByPriceAction;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;

    const CSV_9223: &[u8] = include_bytes!("../../../assets/9223.T.csv");

    #[test]
    fn vec_buy_sell_signal_by_price_action_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_9223);
        let VecBuySellSignalByPriceAction(vec) =
            VecBuySellSignalByPriceAction::from(vec_stock.as_slice());
        assert_eq!(vec.len(), 34);
    }
}
