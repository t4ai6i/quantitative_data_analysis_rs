use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use ta::Next;
use ta::indicators::MovingAverageConvergenceDivergence;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACD {
    pub date: NaiveDate,
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct MACDs<const F: usize, const S: usize, const SG: usize>(Vec<MACD>);

/// # Examples
/// ```
/// use bytes::Bytes;
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
/// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
///
/// tokio_test::block_on(async {
///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
///   let query = queries::get_stocks::Query {
///     ..Default::default()
///   };
///   let stocks = dsv.get_stocks(&query).await.unwrap();
///   let macds = MACDs::<FAST_PERIOD, SLOW_PERIOD, SIGNAL_PERIOD>::from(stocks.as_slice());
///   assert_eq!(macds.len(), 35);
/// });
/// ```
impl<const F: usize, const S: usize, const SG: usize> From<&[Stock]> for MACDs<F, S, SG> {
    fn from(value: &[Stock]) -> Self {
        const {
            assert!(F > 0, "F(fast period) is greater than 0");
        }
        const {
            assert!(S > 0, "S(slow period) is greater than 0");
        }
        const {
            assert!(SG > 0, "SG(signal period) is greater than 0");
        }
        let mut macd = MovingAverageConvergenceDivergence::new(F, S, SG).unwrap();
        let macds = value
            .iter()
            .map(|stock| {
                let (macd, signal, histogram) = macd.next(stock.close).into();
                MACD {
                    date: stock.date,
                    macd,
                    signal,
                    histogram,
                }
            })
            .collect::<Vec<MACD>>();
        MACDs(macds)
    }
}
