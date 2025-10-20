use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum::Display;

use crate::domain::models::stock::model::Stock;

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
pub struct CandleStick {
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
    pub body_ratio: f64,
    pub upper_wick_ratio: f64,
    pub lower_wick_ratio: f64,
    pub bullish_bearish: BullishBearishType,
    pub is_marubozu: bool,
    pub is_doji: bool,
}

impl CandleStick {
    fn get_ratio(numerator: f64, denominator: f64) -> f64 {
        if !denominator.is_normal() {
            return f64::NAN;
        };
        numerator / denominator
    }

    fn get_bullish_bearish_type(open: f64, close: f64) -> BullishBearishType {
        open.partial_cmp(&close)
            .map_or(BullishBearishType::Neither, |ordering| match ordering {
                Ordering::Less => BullishBearishType::Bullish,
                Ordering::Equal => BullishBearishType::Neither,
                Ordering::Greater => BullishBearishType::Bearish,
            })
    }

    // Strict Marubozu: large body, each wick individually very small.
    fn is_marubozu(
        body_ratio: f64,
        body_min_ratio: f64,
        wick_ratio: f64,
        wick_max_ratio: f64,
    ) -> bool {
        if !body_ratio.is_finite() || !wick_ratio.is_finite() {
            return false;
        }
        body_ratio >= body_min_ratio && wick_ratio <= wick_max_ratio
    }

    // Doji: body tiny relative to total range; wick lengths unrestricted.
    fn is_doji(body_ratio: f64, doji_max_body_ratio: f64) -> bool {
        body_ratio.is_finite() && body_ratio <= doji_max_body_ratio
    }
}

struct MarubozuBodyMinRatio(f64);
struct MarubozuWickMaxRatio(f64);
struct DojiMaxBodyRatio(f64);

struct CandleStickInput<'a> {
    stock: &'a Stock,
    marubozu_body_min_ratio: MarubozuBodyMinRatio,
    marubozu_wick_max_ratio: MarubozuWickMaxRatio,
    doji_max_body_ratio: DojiMaxBodyRatio,
}

impl<'a> From<CandleStickInput<'a>> for CandleStick {
    fn from(value: CandleStickInput<'a>) -> Self {
        let CandleStickInput {
            stock,
            marubozu_body_min_ratio,
            marubozu_wick_max_ratio,
            doji_max_body_ratio,
        } = value;
        let Stock {
            date,
            open,
            high,
            low,
            close,
            ..
        } = stock;
        let MarubozuBodyMinRatio(marubozu_body_min_ratio) = marubozu_body_min_ratio;
        let MarubozuWickMaxRatio(marubozu_wick_max_ratio) = marubozu_wick_max_ratio;
        let DojiMaxBodyRatio(doji_max_body_ratio) = doji_max_body_ratio;

        let size = high - low;
        let body = (open - close).abs();
        let body_high = open.max(*close);
        let body_low = open.min(*close);
        let upper_wick = high - body_high;
        let lower_wick = body_low - low;
        let body_ratio = Self::get_ratio(body, size);
        let upper_wick_ratio = Self::get_ratio(upper_wick, size);
        let lower_wick_ratio = Self::get_ratio(lower_wick, size);
        let bullish_bearish = Self::get_bullish_bearish_type(*open, *close);
        let is_marubozu = Self::is_marubozu(
            body_ratio,
            marubozu_body_min_ratio,
            upper_wick_ratio.max(lower_wick_ratio),
            marubozu_wick_max_ratio,
        );
        let is_doji = Self::is_doji(body_ratio, doji_max_body_ratio);
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
            body_ratio,
            upper_wick_ratio,
            lower_wick_ratio,
            bullish_bearish,
            is_marubozu,
            is_doji,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct CandleSticks<const N: usize, const M: usize, const O: usize>(Vec<CandleStick>);

#[derive(thiserror::Error, Debug)]
pub enum CandleSticksError<const N: usize> {
    #[error("MarubozuBodyMinRatio must be less than 100% for Marubozu. but specified {N}%")]
    MarubozuBodyMinRatioGreaterThan100,

    #[error(
        "MarubozuBodyMinRatio must at least 80% for Marubozu and is usually greater than 90%. but specified {N}%"
    )]
    MarubozuBodyMinRatioLessThanEqual80,
}

impl<const N: usize, const M: usize, const O: usize> TryFrom<&[Stock]> for CandleSticks<N, M, O> {
    type Error = CandleSticksError<N>;

    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    ///
    /// use quantitative_data_analysis_rs::domain::models::candle_stick::model::CandleSticks;
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const MARUBOZU_BODY_MIN_RATIO: usize = 90;
    /// const MARUBOZU_WICK_MAX_RATIO: usize = 2;
    /// const DOJI_MAX_BODY_RATIO: usize = 5;
    /// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let candle_sticks = CandleSticks::<MARUBOZU_BODY_MIN_RATIO, MARUBOZU_WICK_MAX_RATIO, DOJI_MAX_BODY_RATIO>::try_from(stocks.as_slice()).unwrap();
    ///   assert_eq!(candle_sticks.len(), 35);
    /// });
    /// ```
    fn try_from(value: &[Stock]) -> Result<Self, Self::Error> {
        if N > 100 {
            return Err(CandleSticksError::MarubozuBodyMinRatioGreaterThan100);
        }
        if N < 80 {
            return Err(CandleSticksError::MarubozuBodyMinRatioLessThanEqual80);
        }
        let body_min_ratio = (N as f64) / 100.0;
        let wick_max_ratio = (M as f64) / 100.0;
        let doji_max_body_ratio = (O as f64) / 100.0;
        let candle_sticks = value
            .par_iter()
            .map(|stock| {
                CandleStick::from(CandleStickInput {
                    stock,
                    marubozu_body_min_ratio: MarubozuBodyMinRatio(body_min_ratio),
                    marubozu_wick_max_ratio: MarubozuWickMaxRatio(wick_max_ratio),
                    doji_max_body_ratio: DojiMaxBodyRatio(doji_max_body_ratio),
                })
            })
            .collect::<Vec<CandleStick>>();
        Ok(CandleSticks(candle_sticks))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::domain::models::candle_stick::model::{
        BullishBearishType, CandleStick, CandleStickInput, DojiMaxBodyRatio, MarubozuBodyMinRatio,
        MarubozuWickMaxRatio,
    };
    use crate::domain::models::stock::model::Stock;

    const MARUBOZU_BODY_MIN_RATIO: usize = 90;
    const MARUBOZU_WICK_MAX_RATIO: usize = 2;
    const DOJI_MAX_BODY_RATIO: usize = 5;

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
        let body_min_ratio = MARUBOZU_BODY_MIN_RATIO as f64 / 100.0;
        let wick_max_ratio = MARUBOZU_WICK_MAX_RATIO as f64 / 100.0;
        let doji_max_body_ratio = DOJI_MAX_BODY_RATIO as f64 / 100.0;
        let actual = CandleStick::from(CandleStickInput {
            stock: &stock,
            marubozu_body_min_ratio: MarubozuBodyMinRatio(body_min_ratio),
            marubozu_wick_max_ratio: MarubozuWickMaxRatio(wick_max_ratio),
            doji_max_body_ratio: DojiMaxBodyRatio(doji_max_body_ratio),
        });
        let expected = CandleStick {
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
            upper_wick_ratio: 0.5,
            lower_wick_ratio: 0.5,
            bullish_bearish: BullishBearishType::Neither,
            is_marubozu: false,
            is_doji: true,
            ..CandleStick::default()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_ratio_test() {
        let actual = CandleStick::get_ratio(500.0, 1000.0);
        assert_eq!(actual, 0.5);

        let actual = CandleStick::get_ratio(1000.0, 1000.0);
        assert_eq!(actual, 1.0);

        // denominator = 0.0 -> is_normal() == false -> returns NaN (仕様)
        let actual = CandleStick::get_ratio(1000.0, 0.0);
        assert!(actual.is_nan());
    }

    #[test]
    fn get_bullish_bearish_type_test() {
        let actual = CandleStick::get_bullish_bearish_type(500.0, 1000.0);
        assert_eq!(actual, BullishBearishType::Bullish);

        let actual = CandleStick::get_bullish_bearish_type(1000.0, 500.0);
        assert_eq!(actual, BullishBearishType::Bearish);

        let actual = CandleStick::get_bullish_bearish_type(1000.0, 1000.0);
        assert_eq!(actual, BullishBearishType::Neither);
    }

    #[test]
    fn is_marubozu_test() {
        let actual = CandleStick::is_marubozu(1.0, 0.9, 0.01, 0.02);
        assert!(actual);

        let actual = CandleStick::is_marubozu(0.9, 0.9, 0.02, 0.02);
        assert!(actual);

        let actual = CandleStick::is_marubozu(0.8, 0.9, 0.02, 0.02);
        assert!(!actual);

        let actual = CandleStick::is_marubozu(0.9, 0.9, 0.03, 0.02);
        assert!(!actual);

        let actual = CandleStick::is_marubozu(0.8, 0.9, 0.03, 0.02);
        assert!(!actual);
    }

    #[test]
    fn is_doji_test() {
        let actual = CandleStick::is_doji(0.04, 0.05);
        assert!(actual);

        let actual = CandleStick::is_doji(0.05, 0.05);
        assert!(actual);

        let actual = CandleStick::is_doji(0.06, 0.05);
        assert!(!actual);
    }
}
