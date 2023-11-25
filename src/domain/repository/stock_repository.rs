use crate::domain::entity::stock::VecStock;
use crate::infrastructure::data_format::DataFormat;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait StockRepository {
    async fn get_vec_stock(
        &self,
        code: impl Into<String> + Send,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format: DataFormat,
    ) -> Result<VecStock>;
}
