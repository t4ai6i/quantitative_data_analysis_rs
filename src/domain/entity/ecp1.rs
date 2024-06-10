use itertools::Itertools;
use std::cmp;

use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
use crate::domain::entity::ordering::Ordering;
use crate::domain::entity::stock::Stock;

pub type CompareLowForTwoDays = Ordering<1, 1>;
pub type CompareHighForTwoDays = Ordering<1, 1>;

pub type ECP1 = (CompareLowForTwoDays, CompareHighForTwoDays);

/// Engulfing Candlestick Pattern1
///
/// ある日とその前日の安値・高値の切り上がり・切り下がりをみる
///
/// * **BuySignal**: 安値切り上がりかつ高値切り上がり
/// * **SellSignal**: 安値切り下がりかつ高値切り下がり
/// * **WaitAndSee**: 上記のどちらにもあてはまならない
impl From<ECP1> for BuySellSignalType {
    fn from(value: ECP1) -> Self {
        let (low, high) = value;
        let low = low.0;
        let high = high.0;
        match (low, high) {
            (Some(cmp::Ordering::Greater), Some(cmp::Ordering::Greater)) => Self::Buy,
            (Some(cmp::Ordering::Less), Some(cmp::Ordering::Less)) => Self::Sell,
            _ => Self::Stay,
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VecECP1(pub Vec<BuySellSignal>);

impl From<&[Stock]> for VecECP1 {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::ecp1::VecECP1;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV_9223: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    ///
    /// let vec_stock = Csv::from_slice::<true>(CSV_9223);
    /// let _ = VecECP1::from(vec_stock.as_slice());
    /// ```
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
                // 前日・当日それぞれの安値を比較する
                let low_trend = Ordering::<1, 1>::from((prev_low, today_low));
                // 前日・当日それぞれの高値を比較する
                let high_trend = Ordering::<1, 1>::from((prev_high, today_high));
                // 高値・安値の切り上げ・切り下げを基にした売買シグナル
                let r#type = BuySellSignalType::from((high_trend, low_trend));
                let date = today.date;
                BuySellSignal { r#type, date }
            })
            .collect_vec();
        VecECP1(vec)
    }
}
