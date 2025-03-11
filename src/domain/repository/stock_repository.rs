use crate::domain::models::stock::model::Stocks;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Params<'a> {
    pub code: &'a str,
    pub market: Option<&'a str>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[async_trait]
pub trait StockRepository {
    async fn get_vec_stock(
        &self,
        code: &str,
        market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Stocks>;
}
