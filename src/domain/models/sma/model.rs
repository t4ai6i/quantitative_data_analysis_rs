use crate::domain::models::close;
use crate::domain::models::stock::model::Stock;
use crate::domain::models::volume;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use ta::Next;
use ta::indicators::SimpleMovingAverage;

/// 終値、取引高の単純移動平均
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA<const N: usize> {
    /// N日目の日付
    pub date: NaiveDate,
    /// 終値のN日単純移動平均
    pub close: close::model::F64,
    /// 取引高のN日単純移動平均
    pub volume: volume::model::F64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct SMAs<const N: usize>(Vec<SMA<N>>);

impl<const N: usize> From<&[Stock]> for SMAs<N> {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    /// use rayon::prelude::*;
    ///
    /// use quantitative_data_analysis_rs::domain::models::sma::model::SMAs;
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const DAYS_5: usize = 5;
    /// const DAYS_25: usize = 25;
    /// const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let smas_5 = SMAs::<DAYS_5>::from(stocks.as_slice());
    ///   assert_eq!(smas_5.len(), 242);
    ///
    ///   let smas_25 = SMAs::<DAYS_25>::from(stocks.as_slice());
    ///   assert_eq!(smas_25.len(), 222);
    ///
    ///   let stocks: Vec<model::Stock> = vec![];
    ///   let smas_5 = SMAs::<DAYS_5>::from(stocks.as_slice());
    ///   assert_eq!(smas_5.len(), 0);
    /// });
    ///
    /// ```
    fn from(value: &[Stock]) -> Self {
        let vec_sma = value
            .par_windows(N)
            .map(|stocks| {
                // 終値のN日と取引高のN日の単純移動平均
                let mut sma_close = SimpleMovingAverage::new(N).unwrap();
                let mut sma_volume = SimpleMovingAverage::new(N).unwrap();
                let mut close = 0.0;
                let mut volume = 0.0;
                for stock in stocks {
                    close = sma_close.next(stock.close);
                    volume = sma_volume.next(stock.volume as f64);
                }
                SMA {
                    date: stocks.last().unwrap().date,
                    close: close::model::F64(close),
                    volume: volume::model::F64(volume),
                }
            })
            .collect::<Vec<SMA<N>>>();
        Self(vec_sma)
    }
}

pub struct SMAPair<'a, const N: usize, const O: usize> {
    pub sma_n: &'a SMA<N>,
    pub sma_o: Option<&'a SMA<O>>,
}

pub struct SMAListPair<'a, const N: usize, const O: usize> {
    pub smas_n: &'a [SMA<N>],
    pub smas_o: &'a [SMA<O>],
}

pub struct SMAListTrio<'a, const N: usize, const O: usize, const P: usize> {
    pub smas_n: &'a [SMA<N>],
    pub smas_o: &'a [SMA<O>],
    pub smas_p: &'a [SMA<P>],
}
