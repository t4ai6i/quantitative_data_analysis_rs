use crate::domain::models::stock::model;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock::repository;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use crate::infrastructure::repositories::stock::structures::internal::csv::Structure;
use anyhow::{Context, Result};
use chrono::NaiveDate;
use rayon::prelude::*;
use std::backtrace::Backtrace;

pub struct Csv {
    pub has_headers: bool,
    pub file_system: FileSystem,
}

impl Csv {
    pub fn new(has_headers: bool, file_system: FileSystem) -> Self {
        Self {
            has_headers,
            file_system,
        }
    }
}
#[async_trait::async_trait]
impl repository::Stock for Csv {
    async fn get_stock(&self, _: &str, _: &str, target_date: NaiveDate) -> Result<model::Stock> {
        let default_string = "".to_string();
        let stocks = self
            .get_stocks(
                default_string.as_str(),
                default_string.as_str(),
                NaiveDate::default(),
                NaiveDate::default(),
            )
            .await?;
        stocks
            .par_iter()
            .find_first(|stock| stock.date.eq(&target_date))
            .cloned()
            .with_context(|| {
                format!(
                    "Not found stock: {}\n{}",
                    target_date,
                    Backtrace::force_capture()
                )
            })
    }

    async fn get_stocks(&self, _: &str, _: &str, _: NaiveDate, _: NaiveDate) -> Result<Stocks> {
        let file = self.file_system.read_file().await?;
        let mut vec_stock = Structure::process_tabular_data(&file, self.has_headers)?;

        let mut stocks = Stocks::default();
        std::mem::swap(&mut vec_stock, &mut stocks);
        Ok(stocks)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::file_system::FileSystem;
    use crate::infrastructure::repositories::stock::file_system::internal::csv::Csv;
    use chrono::NaiveDate;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_stocks_test() -> anyhow::Result<()> {
        let file_path = PathBuf::from("./assets/8473.T.csv");
        let file_system = FileSystem::new(file_path);
        let csv = Csv::new(true, file_system);
        let default_string = "".to_string();
        let stocks = csv
            .get_stocks(
                default_string.as_str(),
                default_string.as_str(),
                NaiveDate::default(),
                NaiveDate::default(),
            )
            .await?;
        assert_eq!(stocks.len(), 246);
        Ok(())
    }
}
