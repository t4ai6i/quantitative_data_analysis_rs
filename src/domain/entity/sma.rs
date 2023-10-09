use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use simple_moving_average::{SumTreeSMA, SMA as OtherSMA};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA<const N_DAY: usize> {
    pub value: f64,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecSMA<const N_DAY: usize>(pub Vec<SMA<N_DAY>>);

impl<const N_DAY: usize> From<&[Stock]> for VecSMA<N_DAY> {
    ///
    /// # Examples
    /// ```ignore
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    /// const FIVE_DAY: usize = 5;
    /// const TWENTY_FIVE_DAY: usize = 25;
    /// let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
    /// let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
    /// assert_eq!(five_days.len(), 242);
    /// let VecSMA(twenty_five_days) = VecSMA::<TWENTY_FIVE_DAY>::from(stocks.as_slice());
    /// assert_eq!(twenty_five_days.len(), 222);
    /// let VecStock(stocks) = VecStock::<true>(vec![]);
    /// let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
    /// assert_eq!(five_days.len(), 0);
    /// ```
    fn from(value: &[Stock]) -> Self {
        let smas = value
            .windows(N_DAY)
            .map(|stocks| {
                let mut ma = SumTreeSMA::<_, f64, { N_DAY }>::new();
                for stock in stocks {
                    ma.add_sample(stock.adj_close);
                }
                let average = ma.get_average();
                let date = stocks.last().unwrap().date;
                SMA {
                    value: average,
                    date,
                }
            })
            .collect_vec();
        Self(smas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::stock::VecStock;
    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const FIVE_DAY: usize = 5;
    const TWENTY_FIVE_DAY: usize = 25;

    #[test]
    fn vec_sma_test() {
        let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
        let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
        assert_eq!(five_days.len(), 242);
        let VecSMA(twenty_five_days) = VecSMA::<TWENTY_FIVE_DAY>::from(stocks.as_slice());
        assert_eq!(twenty_five_days.len(), 222);
        let VecStock(stocks) = VecStock::<true>(vec![]);
        let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
        assert_eq!(five_days.len(), 0);
    }
}
