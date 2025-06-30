use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use simple_moving_average::{SumTreeSMA, SMA as OtherSMA};

/// 終値、取引高の単純移動平均のセット
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMASet<const N: usize> {
    /// 終値
    pub close: f64,
    /// 取引高
    pub volume: f64,
}

impl<const N: usize> From<&[Stock]> for SMASet<N> {
    fn from(value: &[Stock]) -> Self {
        // 終値のN日と取引高のN日の単純移動平均
        let (closes, volumes) = value.iter().fold(
            (
                SumTreeSMA::<_, f64, { N }>::new(),
                SumTreeSMA::<_, f64, { N }>::new(),
            ),
            |acc, stock| {
                let (mut acc_close, mut acc_volume) = acc;
                acc_close.add_sample(stock.close);
                acc_volume.add_sample(stock.volume as _);
                (acc_close, acc_volume)
            },
        );
        let close = closes.get_average();
        let volume = volumes.get_average();
        Self { close, volume }
    }
}

/// 終値、取引高の単純移動平均
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA<const N: usize> {
    /// 終値、取引高のN日単純移動平均
    pub sma_n: SMASet<N>,
    /// N日目の日付
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct SMAs<const N: usize>(Vec<SMA<N>>);

impl<const N: usize> From<&[Stock]> for SMAs<N> {
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use chrono::NaiveDate;
    ///
    /// use quantitative_data_analysis_rs::domain::models::sma::model::SMAs;
    /// use quantitative_data_analysis_rs::domain::models::stock::model::Stocks;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::file_system::FileSystem;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::file_system::internal::csv::Csv;
    ///
    /// const DAYS_5: usize = 5;
    /// const DAYS_25: usize = 25;
    ///
    /// tokio_test::block_on(async {
    ///   let file_path = PathBuf::from("./assets/8473.T.csv");
    ///   let file_system = FileSystem::new(file_path);
    ///   let csv = Csv::new(true, file_system);
    ///   let default_str = "";
    ///   let stocks = csv.get_stocks(default_str, default_str, NaiveDate::default (), NaiveDate::default ()).await.unwrap();
    ///   let smas_5 = SMAs::<DAYS_5>::from(stocks.as_slice());
    ///   assert_eq!(smas_5.len(), 242);
    ///
    ///   let smas_25 = SMAs::<DAYS_25>::from(stocks.as_slice());
    ///   assert_eq!(smas_25.len(), 222);
    ///
    ///   let stocks = Stocks::default();
    ///   let smas_5 = SMAs::<DAYS_5>::from(stocks.as_slice());
    ///   assert_eq!(smas_5.len(), 0);
    /// });
    ///
    /// ```
    fn from(value: &[Stock]) -> Self {
        let vec_sma = value
            .par_windows(N)
            .map(|stocks| SMA {
                sma_n: SMASet::<N>::from(stocks),
                date: stocks.last().unwrap().date,
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
