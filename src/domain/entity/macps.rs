use rayon::prelude::*;

use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
use crate::domain::entity::sma::SMAListTrio;
use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use std::cmp::Ordering;

/// MovingAverageComparisonStrategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACPS {
    pub buy: Option<NaiveDate>,
    pub sell: Option<NaiveDate>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecMACPS(pub Vec<MACPS>);

impl From<(Option<Ordering>, Option<Ordering>, Option<Ordering>)> for BuySellSignalType {
    fn from(value: (Option<Ordering>, Option<Ordering>, Option<Ordering>)) -> Self {
        let (cmp1, cmp2, cmp3) = value;
        match (cmp1, cmp2, cmp3) {
            // Current close < 20-day sma < 50-day sma < 200-day sma → BUY signal
            (Some(Ordering::Less), Some(Ordering::Less), Some(Ordering::Less)) => {
                BuySellSignalType::Buy
            }
            // Current close > 20-day sma > 50-day sma > 200-day sma → SELL signal
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
    /// use quantitative_data_analysis_rs::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
    /// use quantitative_data_analysis_rs::domain::entity::macps::{VecMACPS, MACPS};
    /// use quantitative_data_analysis_rs::domain::entity::sma::{SMAListTrio, VecSMA};
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    ///
    /// let vec_stock = Csv::from_slice::<true>(CSV_8473);
    /// let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
    /// let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
    /// let VecSMA(smas_50) = VecSMA::<50>::from(vec_stock.as_slice());
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
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .last()
            .map(|buy| buy.date);
        let sell = sells
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .last()
            .map(|sell| sell.date);
        Self { buy, sell }
    }
}
