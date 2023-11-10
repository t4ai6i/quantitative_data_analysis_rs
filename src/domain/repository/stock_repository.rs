use crate::domain::entity::stock::VecStock;
use crate::infrastructure::vec_stock_repository::data_format::DataFormatType;
use anyhow::Result;
use chrono::NaiveDate;

pub trait StockRepository {
    fn get_vec_stock(
        &self,
        code: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format_type: DataFormatType,
    ) -> Result<VecStock>;
}
