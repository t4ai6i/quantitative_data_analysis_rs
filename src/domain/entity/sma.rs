use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use simple_moving_average::{SumTreeSMA, SMA as OtherSMA};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA<const N: usize> {
    pub ave: f64,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecSMA<const N: usize>(pub Vec<SMA<N>>);

impl<const N: usize> From<&[Stock]> for VecSMA<N> {
    ///
    /// # Examples
    /// ```ignore
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    /// let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
    /// assert_eq!(smas_5.len(), 242);
    /// let VecSMA(smas_25) = VecSMA::<25>::from(stocks.as_slice());
    /// assert_eq!(smas_25.len(), 222);
    /// let VecStock(stocks) = VecStock::<true>(vec![]);
    /// let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
    /// assert_eq!(smas_5.len(), 0);
    /// ```
    fn from(value: &[Stock]) -> Self {
        let smas = value
            .windows(N)
            .map(|stocks| {
                let mut ma = SumTreeSMA::<_, f64, { N }>::new();
                for stock in stocks {
                    ma.add_sample(stock.close);
                }
                let ave = ma.get_average();
                let date = stocks.last().unwrap().date;
                SMA { ave, date }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::csv_ext::CsvExt;
    use crate::infrastructure::stock_repository::data_format::csv::StockCsvRow;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn vec_sma_test() {
        let vec_stock = StockCsvRow::from_slice::<true>(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
        assert_eq!(smas_5.len(), 242);
        let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
        assert_eq!(smas_25.len(), 222);
        let vec_stock: Vec<Stock> = vec![];
        let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
        assert_eq!(smas_5.len(), 0);
    }
}
