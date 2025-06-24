use rayon::prelude::*;

use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::sma::model::SMAListTrio;
use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use std::cmp::Ordering;

/// MovingAverageComparisonStrategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACPS {
    pub buy: Option<NaiveDate>,
    pub sell: Option<NaiveDate>,
}

impl From<(Option<Ordering>, Option<Ordering>, Option<Ordering>)> for BuySellSignalType {
    fn from(value: (Option<Ordering>, Option<Ordering>, Option<Ordering>)) -> Self {
        let (cmp1, cmp2, cmp3) = value;
        match (cmp1, cmp2, cmp3) {
            // current close < 5day sma < 25day sma < 50day sma → Buy
            (Some(Ordering::Less), Some(Ordering::Less), Some(Ordering::Less)) => {
                BuySellSignalType::Buy
            }
            // current close > 5day sma > 25day sma > 50day sma → Sell
            (Some(Ordering::Greater), Some(Ordering::Greater), Some(Ordering::Greater)) => {
                BuySellSignalType::Sell
            }
            _ => BuySellSignalType::Stay,
        }
    }
}

impl<'a> From<(&[Stock], SMAListTrio<'a, 5, 25, 50>)> for MACPS {
    ///
    /// # Examples
    /// ```
    /// use chrono::NaiveDate;
    /// use rayon::prelude::*;
    /// use quantitative_data_analysis_rs::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
    /// use quantitative_data_analysis_rs::domain::models::macps::model::MACPS;
    /// use quantitative_data_analysis_rs::domain::models::sma::model::{SMAListTrio, SMAs};
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv::Structure;
    ///
    /// const CSV_8473: &[u8] = include_bytes!("../../../../assets/8473.T.csv");
    ///
    /// let successes: Vec<_> = Structure::from_slice::<true>(CSV_8473)
    ///     .into_par_iter().map(|s| s.unwrap()).collect();
    /// let vec_stock: Vec<_> = Structure::from_deserialize(successes)
    ///     .into_par_iter().map(|s| s.unwrap()).collect();
    /// let smas_5 = SMAs::<5>::from(vec_stock.as_slice());
    /// let smas_25 = SMAs::<25>::from(vec_stock.as_slice());
    /// let smas_50 = SMAs::<50>::from(vec_stock.as_slice());
    /// let sma_list_trio = SMAListTrio {
    ///     smas_n: smas_5.as_slice(),
    ///     smas_o: smas_25.as_slice(),
    ///     smas_p: smas_50.as_slice(),
    /// };
    /// let macps = MACPS::from((vec_stock.as_slice(), sma_list_trio));
    /// assert_eq!(macps, MACPS { buy: NaiveDate::from_ymd_opt(2023, 4, 10), sell: NaiveDate::from_ymd_opt(2023, 9, 8) });
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
                let cmp1 = stock.close.partial_cmp(&sma_n.sma_n.close);
                let cmp2 = sma_n.sma_n.close.partial_cmp(&sma_o.sma_n.close);
                let cmp3 = sma_o.sma_n.close.partial_cmp(&sma_p.sma_n.close);
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
