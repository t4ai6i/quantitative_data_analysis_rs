use crate::domain::models::candle_stick::model::{BullishBearishType, CandleSticks};
use crate::shared::custom_date_format::SLASH_DELIMITED_DATE_FORMAT;
use rayon::prelude::*;
use std::ops::Mul;

pub trait CandleSticksExt {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
}

impl<const N: usize> CandleSticksExt for CandleSticks<N> {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::models::candle_stick::model::CandleSticks;
    /// use quantitative_data_analysis_rs::domain::models::stock::model::Stocks;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::data_format::csv::Csv;
    /// use crate::quantitative_data_analysis_rs::presenter::view_models::candle_stick::view_model::CandleSticksExt;
    ///
    /// const CSV_9223: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    /// const MARUBOZU_MIN_RATE: usize = 90;
    ///
    /// let successes: Vec<_> = Csv::from_slice::<true>(CSV_9223)
    ///     .into_iter().map(|s| s.unwrap()).collect();
    /// let vec_stock: Vec<_> = Csv::from_deserialize(successes)
    ///     .into_iter().map(|s| s.unwrap()).collect();
    /// let candle_sticks = CandleSticks::<MARUBOZU_MIN_RATE>::try_from(vec_stock.as_slice()).unwrap();
    /// let _ = candle_sticks.table_chart_rows();
    /// ```
    fn table_chart_rows(&self) -> Vec<Vec<String>> {
        self.par_iter()
            .map(|candle_stick| {
                let date = candle_stick
                    .date
                    .format(SLASH_DELIMITED_DATE_FORMAT.as_str())
                    .to_string();
                let bullish_bearish = match candle_stick.bullish_bearish {
                    BullishBearishType::Neither => "⏹️".to_string(),
                    BullishBearishType::Bullish => "⤴️".to_string(),
                    BullishBearishType::Bearish => "⤵️".to_string(),
                };
                let marubozu = if candle_stick.is_marubozu {
                    "✅".to_string()
                } else {
                    "❌".to_string()
                };
                let doji = if candle_stick.is_doji {
                    "✅".to_string()
                } else {
                    "❌".to_string()
                };
                let body_pct = candle_stick.body_pct.mul(100.0);
                let body_pct = format!("{:.1}", body_pct);
                let size = format!(
                    "{}, {}({}%)",
                    candle_stick.size, candle_stick.body, body_pct
                );
                let upper_wick_pct = candle_stick.upper_wick_pct.mul(100.0);
                let upper_wick_pct = format!("{:.1}", upper_wick_pct);
                let lower_wick_pct = candle_stick.lower_wick_pct.mul(100.0);
                let lower_wick_pct = format!("{:.1}", lower_wick_pct);
                let wick = format!(
                    "{}({}%), {}({}%)",
                    candle_stick.upper_wick,
                    upper_wick_pct,
                    candle_stick.lower_wick,
                    lower_wick_pct,
                );
                vec![date, size, bullish_bearish, marubozu, wick, doji]
            })
            .collect()
    }
}
