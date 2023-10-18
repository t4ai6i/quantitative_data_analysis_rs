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

pub trait VecSMAExt {
    fn collect_vec_ave(&self) -> Vec<f32>;
}

impl<const N: usize> VecSMAExt for VecSMA<N> {
    fn collect_vec_ave(&self) -> Vec<f32> {
        self.0.iter().map(|sma| sma.ave as f32).collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::stock::VecStock;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn vec_sma_test() {
        let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
        assert_eq!(smas_5.len(), 242);
        let VecSMA(smas_25) = VecSMA::<25>::from(stocks.as_slice());
        assert_eq!(smas_25.len(), 222);
        let VecStock(stocks) = VecStock::<true>(vec![]);
        let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
        assert_eq!(smas_5.len(), 0);
    }
}
