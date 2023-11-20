use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::csv_ext::CsvExt;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::stock_repository::data_format::csv::StockCsvRow;
use crate::infrastructure::stock_repository::data_format::DataFormat;
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
impl StockRepository for FileSystem {
    ///
    /// # Examples
    /// ```ignore
    /// let repository = FileSystem::new(PathBuf::from("./assets/"));
    /// let code = "8473.T";
    /// let start_date = NaiveDate::default();
    /// let end_date = NaiveDate::default();
    /// let data_format = DataFormatType::CSV { has_headers: true };
    /// let vec_stock = repository
    ///     .get_vec_stock(code.to_string(), start_date, end_date, data_format)
    ///     .await?;
    /// assert_eq!(vec_stock.0.len(), 246);
    async fn get_vec_stock(
        &self,
        code: impl Into<String> + Send,
        _: NaiveDate,
        _: NaiveDate,
        data_format: DataFormat,
    ) -> Result<VecStock> {
        if let DataFormat::CSV { has_headers } = data_format {
            let filename = format!("{}.csv", code.into());
            let path = self.root.join(filename);
            let csv = tokio::fs::read(path).await?;
            let vec_stock = if has_headers {
                StockCsvRow::from_slice::<true>(csv.as_slice())
            } else {
                StockCsvRow::from_slice::<false>(csv.as_slice())
            };
            Ok(VecStock(vec_stock))
        } else {
            bail!("Unsupported data format: {:?}", data_format);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::stock_repository::data_format::DataFormat;
    use crate::infrastructure::stock_repository::file_system::FileSystem;
    use anyhow::Result;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let repository = FileSystem::new(PathBuf::from("./assets/"));
        let code = "8473.T";
        let start_date = NaiveDate::default();
        let end_date = NaiveDate::default();
        let data_format = DataFormat::CSV { has_headers: true };
        let vec_stock = repository
            .get_vec_stock(code, start_date, end_date, data_format)
            .await?;
        assert_eq!(vec_stock.0.len(), 246);
        Ok(())
    }
}
