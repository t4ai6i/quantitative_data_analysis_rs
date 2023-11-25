use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::csv_ext::CsvExt;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::stock_repository::data_format::csv::StockCsvRow;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use std::backtrace::Backtrace;
use tokio::fs::read;

#[async_trait]
impl StockRepository for FileSystem {
    ///
    /// # Examples
    /// ```ignore
    /// let code = "8473.T";
    /// let start_date = NaiveDate::default();
    /// let end_date = NaiveDate::default();
    /// let file_path = PathBuf::from(format!("./assets/{}.csv", code));
    /// let data_format = DataFormat::CSV {
    ///     has_headers: true,
    ///     file_path,
    /// };
    /// let repository = FileSystem::new(data_format);
    /// let vec_stock = repository.get_vec_stock(code, start_date, end_date).await?;
    /// assert_eq!(vec_stock.0.len(), 246);
    async fn get_vec_stock(
        &self,
        _: impl Into<String> + Send,
        _: NaiveDate,
        _: NaiveDate,
    ) -> Result<VecStock> {
        let file_path = if let DataFormat::CSV { ref file_path, .. } = self.data_format {
            file_path
        } else {
            bail!(format!(
                "Unsupported data format: {:?}\n{}",
                self.data_format,
                Backtrace::force_capture()
            ));
        };
        let file = read(file_path).await.with_context(|| {
            format!(
                "File not found: {:?}). \n{}",
                file_path,
                Backtrace::force_capture()
            )
        })?;
        let vec_stock = if let DataFormat::CSV { has_headers, .. } = self.data_format {
            if has_headers {
                StockCsvRow::from_slice::<true>(file.as_slice())
            } else {
                StockCsvRow::from_slice::<false>(file.as_slice())
            }
        } else {
            vec![]
        };
        Ok(VecStock(vec_stock))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::stock_repository::file_system::FileSystem;
    use anyhow::Result;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let code = "8473.T";
        let start_date = NaiveDate::default();
        let end_date = NaiveDate::default();
        let file_path = PathBuf::from(format!("./assets/{}.csv", code));
        let data_format = DataFormat::CSV {
            has_headers: true,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let vec_stock = repository.get_vec_stock(code, start_date, end_date).await?;
        assert_eq!(vec_stock.0.len(), 246);
        Ok(())
    }
}
