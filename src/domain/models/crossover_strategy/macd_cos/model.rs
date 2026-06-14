use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::macd::model::{MACD, MACDs};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;

/// MACD Crossover Strategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MacdCos {
    pub date: NaiveDate,
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
    pub crossover_pattern: CrossoverPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct MacdCoses(Vec<MacdCos>);

impl<const F: usize, const S: usize, const SG: usize> From<MACDs<F, S, SG>> for MacdCoses {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    /// use chrono::NaiveDate;
    /// use rayon::prelude::*;
    ///
    /// use quantitative_data_analysis_rs::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
    /// use quantitative_data_analysis_rs::domain::models::crossover_strategy::macd_cos::model::{ MacdCos, MacdCoses };
    /// use quantitative_data_analysis_rs::domain::models::macd::model::MACDs;
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const FAST_PERIOD: usize = 12;
    /// const SLOW_PERIOD: usize = 26;
    /// const SIGNAL_PERIOD: usize = 9;
    /// const CSV: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let macds = MACDs::<FAST_PERIOD, SLOW_PERIOD, SIGNAL_PERIOD>::from(stocks.as_slice());
    ///   let macd_coses = MacdCoses::from(macds);
    ///   assert_eq!(macd_coses.len(), 245);
    /// });
    /// ```
    fn from(value: MACDs<F, S, SG>) -> Self {
        let vec_macd_cos = value
            .as_slice()
            .par_windows(2)
            .map(|windows| {
                // 昨日と今日のMACDとシグナルの比較からクロスオーバーパターンを判定
                let yesterday = windows[0].macd.partial_cmp(&windows[0].signal);
                let target = windows[1].macd.partial_cmp(&windows[1].signal);
                let crossover_pattern = CrossoverPattern::from((yesterday, target));
                let MACD {
                    date,
                    macd,
                    signal,
                    histogram,
                } = windows[1];
                MacdCos {
                    date,
                    macd,
                    signal,
                    histogram,
                    crossover_pattern,
                }
            })
            .collect::<Vec<MacdCos>>();
        Self(vec_macd_cos)
    }
}
