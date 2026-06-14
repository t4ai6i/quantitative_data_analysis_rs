use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::sma::model::SMAListTrio;
use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use rayon::prelude::*;
use std::cmp::Ordering;

/// SimpleMovingAverageComparisonStrategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SmaCps {
    pub buy: Option<NaiveDate>,
    pub sell: Option<NaiveDate>,
}

impl From<(Option<Ordering>, Option<Ordering>, Option<Ordering>)> for BuySellSignalType {
    fn from(value: (Option<Ordering>, Option<Ordering>, Option<Ordering>)) -> Self {
        let (cmp1, cmp2, cmp3) = value;
        match (cmp1, cmp2, cmp3) {
            // current close < 5day SMA < 25day SMA < 50day SMA → Buy
            (Some(Ordering::Less), Some(Ordering::Less), Some(Ordering::Less)) => {
                BuySellSignalType::Buy
            }
            // current close > 5day SMA > 25day SMA > 50day SMA → Sell
            (Some(Ordering::Greater), Some(Ordering::Greater), Some(Ordering::Greater)) => {
                BuySellSignalType::Sell
            }
            _ => BuySellSignalType::Stay,
        }
    }
}

impl<'a> From<(&[Stock], SMAListTrio<'a, 5, 25, 50>)> for SmaCps {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    /// use chrono::NaiveDate;
    /// use rayon::prelude::*;
    ///
    /// use quantitative_data_analysis_rs::domain::models::crossover_strategy::sma_cps::model::SmaCps;
    /// use quantitative_data_analysis_rs::domain::models::sma::model::{SMAListTrio, SMAs};
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const CSV: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let smas_5 = SMAs::<5>::from(stocks.as_slice());
    ///   let smas_25 = SMAs::<25>::from(stocks.as_slice());
    ///   let smas_50 = SMAs::<50>::from(stocks.as_slice());
    ///   let sma_list_trio = SMAListTrio {
    ///       smas_n: smas_5.as_slice(),
    ///       smas_o: smas_25.as_slice(),
    ///       smas_p: smas_50.as_slice(),
    ///   };
    ///   let sma_cps = SmaCps::from((stocks.as_slice(), sma_list_trio));
    ///   assert_eq!(sma_cps, SmaCps { buy: NaiveDate::from_ymd_opt(2023, 4, 10), sell: NaiveDate::from_ymd_opt(2023, 9, 8) });
    /// });
    /// ```
    fn from(value: (&[Stock], SMAListTrio<'a, 5, 25, 50>)) -> Self {
        let (
            stocks,
            SMAListTrio {
                smas_n,
                smas_o,
                smas_p,
            },
        ) = value;
        let (buys, sells): (Vec<_>, Vec<_>) = stocks
            .par_iter()
            .filter_map(|stock| {
                let date = stock.date;
                let sma_n = smas_n.par_iter().find_first(|sma_n| sma_n.date.eq(&date))?;
                let sma_o = smas_o.par_iter().find_first(|sma_o| sma_o.date.eq(&date))?;
                let sma_p = smas_p.par_iter().find_first(|sma_p| sma_p.date.eq(&date))?;
                let cmp1 = stock.close.partial_cmp(&sma_n.close.0);
                let cmp2 = sma_n.close.0.partial_cmp(&sma_o.close.0);
                let cmp3 = sma_o.close.0.partial_cmp(&sma_p.close.0);
                let r#type = BuySellSignalType::from((cmp1, cmp2, cmp3));
                let r#type = r#type.ne(&BuySellSignalType::Stay).then_some(r#type)?;
                Some(BuySellSignal { date, r#type })
            })
            .partition(|buy_sell_signal| buy_sell_signal.r#type.eq(&BuySellSignalType::Buy));
        let buy = buys
            .par_iter()
            .max_by(|a, b| Ord::cmp(&a.date, &b.date))
            .map(|buy| buy.date);
        let sell = sells
            .par_iter()
            .max_by(|a, b| Ord::cmp(&a.date, &b.date))
            .map(|sell| sell.date);
        Self { buy, sell }
    }
}
