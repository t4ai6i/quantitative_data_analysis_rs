use crate::domain::entity::buy_sell_signal::BuySellSignalType;
use crate::domain::entity::ordering::Ordering;
use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;

/// Engulfing Candlestick Pattern
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct EngulfingCandlestickPattern {
    /// 前日・当日それぞの高値が切り上がったか切り下がったか
    high_trend: Ordering<1, 1>,
    /// 前日・当日それぞの安値が切り上がったか切り下がったか
    low_trend: Ordering<1, 1>,
    /// 高値・安値の切り上げ・切り下げを基にした売買シグナル
    pub(crate) buy_sell_signal: BuySellSignalType,
    /// シグナルの対象の日付
    pub(crate) date: NaiveDate,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct VecEngulfingCandlestickPattern(pub Vec<EngulfingCandlestickPattern>);

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
                let low_trend = Ordering::<1, 1>::from((prev_low, today_low));
                let high_trend = Ordering::<1, 1>::from((prev_high, today_high));
                let buy_sell_signal = BuySellSignalType::from((high_trend, low_trend));
                let date = today.date;
                EngulfingCandlestickPattern {
                    high_trend,
                    low_trend,
                    buy_sell_signal,
                    date,
                }
            })
            .collect_vec();
        VecEngulfingCandlestickPattern(vec)
    }
}
