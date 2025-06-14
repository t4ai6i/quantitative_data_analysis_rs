use crate::domain::models::stock::model::{Stock, Stocks};
use crate::domain::repositories::stock::repository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use crate::infrastructure::repositories::stock::data_format::csv::Csv;
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use std::backtrace::Backtrace;

#[async_trait]
impl repository::Stock for FileSystem {
    async fn get_stock(&self, code: &str, market: &str, target_date: NaiveDate) -> Result<Stock> {
        todo!()
    }

    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use chrono::NaiveDate;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::data_format::DataFormat;
    /// use quantitative_data_analysis_rs::infrastructure::file_system::FileSystem;
    ///
    /// #[tokio::main] // Add an async runtime for executing asynchronous code
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let code = "8473";
    ///     let market = "T";
    ///     let start_date = NaiveDate::default();
    ///     let end_date = NaiveDate::default();
    ///     let file_path = PathBuf::from(format!("./assets/{}.{}.csv", code, market));
    ///     let data_format = DataFormat::CSV {
    ///         has_headers: true,
    ///         file_path,
    ///     };
    ///     let repository = FileSystem::new(data_format);
    ///     let stocks = repository.get_stocks(code, market, start_date, end_date).await?;
    ///     assert_eq!(stocks.len(), 246);
    ///     Ok(())
    /// }
    /// ```
    async fn get_stocks(&self, _: &str, _: &str, _: NaiveDate, _: NaiveDate) -> Result<Stocks> {
        let file = self.read_file().await?;
        let mut vec_stock = match self.data_format {
            DataFormat::CSV { has_headers, .. } if has_headers => {
                Csv::process_tabular_data(&file, has_headers)?
            }
            _ => bail!(
                "Unsupported data format: {:?}\n{}",
                self.data_format,
                Backtrace::force_capture()
            ),
        };

        let mut stocks = Stocks::default();
        std::mem::swap(&mut vec_stock, &mut stocks);
        Ok(stocks)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::repositories::stock::file_system::FileSystem;
    use anyhow::Result;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_stocks_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let start_date = NaiveDate::default();
        let end_date = NaiveDate::default();
        let file_path = PathBuf::from(format!("./assets/{}.{}.csv", code, market));
        let data_format = DataFormat::CSV {
            has_headers: true,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let stocks = repository
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 246);
        Ok(())
    }
}
