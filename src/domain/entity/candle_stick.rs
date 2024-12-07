use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::ops::{Div, Sub};
use strum::Display;

/// Bullish or Bearish
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum BullishBearishType {
    #[default]
    Neither,
    Bullish,
    Bearish,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct CandleStick<const N: usize> {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub size: f64,
    pub body: f64,
    pub body_high: f64,
    pub body_low: f64,
    pub upper_wick: f64,
    pub lower_wick: f64,
    pub body_pct: f64,
    pub upper_wick_pct: f64,
    pub lower_wick_pct: f64,
    pub bullish_bearish: BullishBearishType,
    pub is_marubozu: bool,
    pub is_doji: bool,
}

impl<const N: usize> CandleStick<N> {
    fn get_size(high: f64, low: f64) -> f64 {
        high - low
    }

    fn get_body(open: f64, close: f64) -> f64 {
        open.sub(&close).abs()
    }

    fn get_upper_wick(high: f64, body_high: f64) -> f64 {
        high - body_high
    }

    fn get_lower_wick(low: f64, body_low: f64) -> f64 {
        body_low - low
    }

    fn get_percentage(size: f64, target: f64) -> f64 {
        if !size.is_normal() {
            return 1.0;
        }
        target / size
    }

    fn get_bullish_bearish_type(open: f64, close: f64) -> BullishBearishType {
        open.partial_cmp(&close)
            .map_or(BullishBearishType::Neither, |ordering| match ordering {
                Ordering::Less => BullishBearishType::Bullish,
                Ordering::Equal => BullishBearishType::Neither,
                Ordering::Greater => BullishBearishType::Bearish,
            })
    }

    fn is_marubozu(body_pct: f64, min_body_pct: f64) -> bool {
        body_pct >= min_body_pct
    }

    fn is_doji(body: f64, upper_wick: f64, lower_wick: f64) -> bool {
        upper_wick > body && lower_wick > body
    }
}

impl<const N: usize> From<&Stock> for CandleStick<N> {
    fn from(value: &Stock) -> Self {
        assert!(
            N <= 100,
            "Minimum Body Percent must be less than 100 for Marubozu (<=100%)."
        );
        assert!(
            N >= 80,
            "Minimum Body Percent must at least 80% for Marubozu and is usually greater than 90%."
        );
        let min_body_pct = N.to_f64().unwrap().div(100.0);

        let Stock {
            date,
            open,
            high,
            low,
            close,
            ..
        } = value;
        let size = Self::get_size(*high, *low);
        let body = Self::get_body(*open, *close);
        let body_high = open.max(*close);
        let body_low = open.min(*close);
        let upper_wick = Self::get_upper_wick(*high, body_high);
        let lower_wick = Self::get_lower_wick(*low, body_low);
        let body_pct = Self::get_percentage(size, body);
        let upper_wick_pct = Self::get_percentage(size, upper_wick);
        let lower_wick_pct = Self::get_percentage(size, lower_wick);
        let bullish_bearish = Self::get_bullish_bearish_type(*open, *close);
        let is_marubozu = Self::is_marubozu(body_pct, min_body_pct);
        let is_doji = Self::is_doji(body, upper_wick, lower_wick);
        Self {
            date: *date,
            open: *open,
            high: *high,
            low: *low,
            close: *close,
            size,
            body,
            body_high,
            body_low,
            upper_wick,
            lower_wick,
            body_pct,
            upper_wick_pct,
            lower_wick_pct,
            bullish_bearish,
            is_marubozu,
            is_doji,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecCandleStick<const N: usize>(pub Vec<CandleStick<N>>);

impl<const N: usize> From<&[Stock]> for VecCandleStick<N> {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::candle_stick::VecCandleStick;
    /// use quantitative_data_analysis_rs::domain::entity::stock::VecStock;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    ///
    /// let stocks  = Csv::from_slice::<true>(CSV);
    /// let VecCandleStick(candle_sticks) = VecCandleStick::<90>::from(stocks.as_slice());
    /// assert_eq!(candle_sticks.len(), 35);
    /// ```
    fn from(value: &[Stock]) -> Self {
        let candle_sticks = value.iter().map(CandleStick::<N>::from).collect_vec();
        VecCandleStick(candle_sticks)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::candle_stick::{BullishBearishType, CandleStick};
    use crate::domain::entity::stock::Stock;

    #[test]
    fn new_test() {
        let stock = Stock {
            date: Default::default(),
            open: 1000.0,
            high: 1500.0,
            low: 500.0,
            close: 1000.0,
            adj_close: 1000.0,
            volume: 10000,
        };
        let actual = CandleStick::<90>::from(&stock);
        let expected = CandleStick::<90> {
            size: 1000.0,
            open: 1000.0,
            high: 1500.0,
            low: 500.0,
            close: 1000.0,
            body: 0.0,
            body_high: 1000.0,
            body_low: 1000.0,
            upper_wick: 500.0,
            lower_wick: 500.0,
            upper_wick_pct: 0.5,
            lower_wick_pct: 0.5,
            bullish_bearish: BullishBearishType::Neither,
            is_marubozu: false,
            is_doji: true,
            ..CandleStick::<90>::default()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_size_test() {
        let actual = CandleStick::<90>::get_size(1000.0, 500.0);
        assert_eq!(actual, 500.0);
    }

    #[test]
    fn get_body_test() {
        let actual = CandleStick::<90>::get_body(1000.0, 500.0);
        assert_eq!(actual, 500.0);

        let actual = CandleStick::<90>::get_body(500.0, 1000.0);
        assert_eq!(actual, 500.0);
    }

    #[test]
    fn get_upper_wick_test() {
        let actual = CandleStick::<90>::get_upper_wick(1000.0, 500.0);
        assert_eq!(actual, 500.0);

        let actual = CandleStick::<90>::get_upper_wick(1000.0, 500.0);
        assert_eq!(actual, 500.0);
    }

    #[test]
    fn get_lower_wick_test() {
        let actual = CandleStick::<90>::get_lower_wick(500.0, 600.0);
        assert_eq!(actual, 100.0);

        let actual = CandleStick::<90>::get_lower_wick(100.0, 500.0);
        assert_eq!(actual, 400.0);
    }

    #[test]
    fn get_percentage_in_size_test() {
        let actual = CandleStick::<90>::get_percentage(1000.0, 500.0);
        assert_eq!(actual, 0.5);

        let actual = CandleStick::<90>::get_percentage(1000.0, 0.0);
        assert_eq!(actual, 0.0);

        let actual = CandleStick::<90>::get_percentage(1000.0, 1000.0);
        assert_eq!(actual, 1.0);

        let actual = CandleStick::<90>::get_percentage(0.0, 500.0);
        assert_eq!(actual, 1.0);
    }

    #[test]
    fn get_bullish_bearish_type_test() {
        let actual = CandleStick::<90>::get_bullish_bearish_type(500.0, 1000.0);
        assert_eq!(actual, BullishBearishType::Bullish);

        let actual = CandleStick::<90>::get_bullish_bearish_type(1000.0, 500.0);
        assert_eq!(actual, BullishBearishType::Bearish);

        let actual = CandleStick::<90>::get_bullish_bearish_type(1000.0, 1000.0);
        assert_eq!(actual, BullishBearishType::Neither);
    }

    #[test]
    fn is_marubozu_test() {
        let actual = CandleStick::<90>::is_marubozu(1.0, 0.9);
        assert!(actual);

        let actual = CandleStick::<90>::is_marubozu(0.9, 0.9);
        assert!(actual);

        let actual = CandleStick::<90>::is_marubozu(0.8, 0.9);
        assert!(!actual);
    }

    #[test]
    fn is_doji_test() {
        let actual = CandleStick::<90>::is_doji(500.0, 501.0, 501.0);
        assert!(actual);

        let actual = CandleStick::<90>::is_doji(500.0, 500.0, 500.0);
        assert!(!actual);

        let actual = CandleStick::<90>::is_doji(500.0, 501.0, 500.0);
        assert!(!actual);

        let actual = CandleStick::<90>::is_doji(500.0, 500.0, 501.0);
        assert!(!actual);
    }
}
