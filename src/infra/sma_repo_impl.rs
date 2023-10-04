use crate::domain::entity::sma::SMA as StructSMA;
use crate::domain::entity::stock::Stock;
use crate::domain::repo::sma_repo::SMARepo;
use itertools::Itertools;
use simple_moving_average::{SumTreeSMA, SMA};

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct SMARepoImpl;

impl SMARepoImpl {
    pub fn new() -> Self {
        Self
    }
}

impl SMARepo for SMARepoImpl {
    ///
    /// # Examples
    /// ```ignore
    /// let sma_repo = SMARepoImpl::new();
    /// let five_day_sma = sma_repo.collect_vec::<FIVE_DAY>(&stocks);
    /// assert_eq!(five_day_sma.len(), 242);
    /// ```
    fn collect_vec<const N_DAY: usize>(&self, stocks: &[Stock]) -> Vec<StructSMA> {
        stocks
            .windows(N_DAY)
            .map(|stocks| {
                let mut ma = SumTreeSMA::<_, f64, { N_DAY }>::new();
                for stock in stocks {
                    ma.add_sample(stock.adj_close);
                }
                let average = ma.get_average();
                let date = stocks.last().unwrap().date;
                StructSMA {
                    value: average,
                    date,
                }
            })
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repo::stock_repo::StockRepo;
    use crate::infra::stock_repo_impl::StockRepoImpl;
    use chrono::NaiveDate;

    static CSV_8473: &[u8] = include_bytes!("../../assets/8473.T.csv");

    const FIVE_DAY: usize = 5;
    const TWENTY_FIVE_DAY: usize = 25;

    #[test]
    fn sma_repo_impl_collect_vec() {
        let stock_repo = StockRepoImpl::new();
        let stocks = stock_repo.vec_from_csv(CSV_8473, true);
        let sma_repo = SMARepoImpl::new();
        let five_day_smas = sma_repo.collect_vec::<FIVE_DAY>(&stocks);
        assert_eq!(five_day_smas.len(), 242);
        let first = five_day_smas.first().unwrap();
        assert_eq!(
            first,
            &StructSMA {
                value: 2554.3865234,
                date: NaiveDate::from_ymd_opt(2022, 9, 15).unwrap(),
            }
        );
        let twenty_five_day_smas = sma_repo.collect_vec::<TWENTY_FIVE_DAY>(&stocks);
        assert_eq!(twenty_five_day_smas.len(), 222);
        let first = twenty_five_day_smas.first().unwrap();
        assert_eq!(
            first,
            &StructSMA {
                value: 2513.62647464,
                date: NaiveDate::from_ymd_opt(2022, 10, 18).unwrap(),
            }
        );
    }
}
