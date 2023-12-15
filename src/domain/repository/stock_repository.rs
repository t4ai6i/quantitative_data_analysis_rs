use crate::domain::entity::stock::VecStock;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait StockRepository {
    async fn get_vec_stock(
        &self,
        code: &str,
        market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<VecStock>;
}
