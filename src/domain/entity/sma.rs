use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use simple_moving_average::{SumTreeSMA, SMA as OtherSMA};

/// 終値、取引高の平均値
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Average<const N: usize> {
    /// 終値
    pub close: f64,
    /// 取引高
    pub volume: f64,
}

impl<const N: usize> From<&[Stock]> for Average<N> {
    fn from(value: &[Stock]) -> Self {
        // 終値のN日の単純移動平均
        let mut ma = SumTreeSMA::<_, f64, { N }>::new();
        for stock in value {
            ma.add_sample(stock.close);
        }
        let close = ma.get_average();

        // 取引高のN日の単純移動平均
        let mut ma = SumTreeSMA::<_, f64, { N }>::new();
        for stock in value {
            ma.add_sample(stock.volume as _);
        }
        let volume = ma.get_average();
        Self { close, volume }
    }
}

/// 終値、取引高の単純移動平均
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA<const N: usize> {
    /// 終値、取引高のN日単純移動平均
    pub average: Average<N>,
    /// N日目の日付
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecSMA<const N: usize>(pub Vec<SMA<N>>);

impl<const N: usize> From<&[Stock]> for VecSMA<N> {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    /// use quantitative_data_analysis_rs::domain::entity::sma::VecSMA;
    /// use quantitative_data_analysis_rs::domain::entity::stock::VecStock;
    ///
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    /// const DAYS_5: usize = 5;
    /// const DAYS_25: usize = 25;
    ///
    /// let stocks = Csv::from_slice::<true>(CSV_8473);
    /// let VecSMA(smas_5) = VecSMA::<DAYS_5>::from(stocks.as_slice());
    /// assert_eq!(smas_5.len(), 242);
    ///
    /// let VecSMA(smas_25) = VecSMA::<DAYS_25>::from(stocks.as_slice());
    /// assert_eq!(smas_25.len(), 222);
    ///
    /// let VecStock(stocks) = VecStock(vec![]);
    /// let VecSMA(smas_5) = VecSMA::<DAYS_5>::from(stocks.as_slice());
    /// assert_eq!(smas_5.len(), 0);
    /// ```
    fn from(value: &[Stock]) -> Self {
        let smas = value
            .windows(N)
            .map(|stocks| SMA {
                average: Average::<N>::from(stocks),
                date: stocks.last().unwrap().date,
            })
            .collect_vec();
        Self(smas)
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
