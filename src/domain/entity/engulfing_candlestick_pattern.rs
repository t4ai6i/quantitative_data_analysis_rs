use crate::domain::entity::buy_sell_signal::BuySellSignalType;
use crate::domain::entity::ordering::Ordering;
use crate::domain::entity::stock::Stock;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignal;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// Engulfing Candlestick Pattern
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct VecEngulfingCandlestickPattern(pub Vec<BuySellSignal>);

impl From<&[Stock]> for VecEngulfingCandlestickPattern {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::engulfing_candlestick_pattern::VecEngulfingCandlestickPattern;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV_9223: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    ///
    /// let vec_stock = Csv::from_slice::<true>(CSV_9223);
    /// let VecEngulfingCandlestickPattern(vec) =
    ///     VecEngulfingCandlestickPattern::from(vec_stock.as_slice());
    /// assert_eq!(vec.len(), 34);
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
                let buy_sell_signal = BuySellSignalType::from((high_trend, low_trend));
                let date = today.date;
                BuySellSignal {
                    buy_sell_signal,
                    date,
                }
            })
            .collect_vec();
        VecEngulfingCandlestickPattern(vec)
    }
}
