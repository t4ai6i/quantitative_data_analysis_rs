use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock_repository::StockRepository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use crate::infrastructure::stock_repository::data_format::csv::Csv;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use std::backtrace::Backtrace;
use tokio::fs::read;

#[async_trait]
impl StockRepository for FileSystem {
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use chrono::NaiveDate;
    /// use quantitative_data_analysis_rs::domain::repositories::stock_repository::StockRepository;
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
        let DataFormat::CSV { ref file_path, .. } = self.data_format else {
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
        let vec_stock = match self.data_format {
            DataFormat::CSV { has_headers, .. } if has_headers => {
                Csv::from_slice::<true>(file.as_slice())
            }
            DataFormat::CSV { has_headers, .. } if !has_headers => {
                Csv::from_slice::<false>(file.as_slice())
            }
            _ => vec![],
        };
        let mut stocks = Stocks::default();
        stocks.extend(vec_stock);

        Ok(stocks)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::stock_repository::StockRepository;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::stock_repository::file_system::FileSystem;
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
