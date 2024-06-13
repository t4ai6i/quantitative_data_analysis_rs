use crate::domain::entity::candle_stick::{BullishBearishType, VecCandleStick};
use itertools::Itertools;
use std::ops::Mul;

pub trait VecCandleStickExt {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
}

impl<const N: usize> VecCandleStickExt for VecCandleStick<N> {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::candle_stick::VecCandleStick;
    /// use quantitative_data_analysis_rs::domain::entity::stock::VecStock;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    /// use crate::quantitative_data_analysis_rs::presenter::view_model::vec_candle_stick::VecCandleStickExt;
    ///
    /// const CSV: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    ///
    /// let stocks  = Csv::from_slice::<true>(CSV);
    /// let vec_candle_stick = VecCandleStick::<90>::from(stocks.as_slice());
    /// let _ = vec_candle_stick.table_chart_rows();
    /// ```
    fn table_chart_rows(&self) -> Vec<Vec<String>> {
        self.0
            .iter()
            .map(|candle_stick| {
                let date = candle_stick.date.format("%Y/%m/%d").to_string();
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
            .collect_vec()
    }
}
