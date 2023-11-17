use crate::domain::entity::stock::VecStock;
use crate::infrastructure::stock_repository::data_format::DataFormatType;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait StockRepository {
    async fn get_vec_stock(
        &self,
        code: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format_type: DataFormatType,
    ) -> Result<VecStock>;
}
