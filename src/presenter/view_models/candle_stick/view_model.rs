use std::ops::Mul;

use rayon::prelude::*;

use crate::domain::models::candle_stick::model;
use crate::domain::models::candle_stick::model::BullishBearishType;
use crate::shared::custom_date_format::SLASH_DELIMITED_DATE_FORMAT;

pub trait CandleSticks {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
}

impl<const N: usize> CandleSticks for model::CandleSticks<N> {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::models::candle_stick::model;
    /// use quantitative_data_analysis_rs::domain::models::stock::model::Stocks;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    /// use quantitative_data_analysis_rs::presenter::view_models::candle_stick::view_model::CandleSticks;
    ///
    /// const MARUBOZU_MIN_RATE: usize = 90;
    /// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, CSV.to_vec());
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(query).await.unwrap();
    ///   let candle_sticks = model::CandleSticks::<MARUBOZU_MIN_RATE>::try_from(stocks.as_slice()).unwrap();
    ///   let table_chart_rows = candle_sticks.table_chart_rows();
    ///   assert_eq!(table_chart_rows.len(), 35);
    /// });
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
