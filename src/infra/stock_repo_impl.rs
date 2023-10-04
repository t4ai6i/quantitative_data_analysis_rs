use crate::domain::entity::stock::Stock;
use crate::domain::repo::stock_repo::StockRepo;
use csv::ReaderBuilder;
use itertools::Itertools;

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct StockRepoImpl;

impl StockRepoImpl {
    pub fn new() -> Self {
        Self
    }
}

impl StockRepo for StockRepoImpl {
    ///
    /// # Examples
    /// ```ignore
    /// let repo = StockRepositoryImpl::new();
    /// let stocks = repo.list_from_csv(CSV_8473, true);
    /// assert_eq!(stocks.len(), 246);
    /// ```
    fn vec_from_csv<R: std::io::Read>(&self, rdr: R, has_headers: bool) -> Vec<Stock> {
        let mut reader = ReaderBuilder::new()
            .has_headers(has_headers)
            .from_reader(rdr);
        let stocks = reader
            .deserialize::<Stock>()
            .filter_map(Result::ok)
            .collect_vec();
        stocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static CSV_8473: &[u8] = include_bytes!("../../assets/8473.T.csv");

    #[test]
    fn stock_repo_impl_list_from_csv() {
        let repo = StockRepoImpl::new();
        let stocks = repo.vec_from_csv(CSV_8473, true);
        assert_eq!(stocks.len(), 246);
    }
}
