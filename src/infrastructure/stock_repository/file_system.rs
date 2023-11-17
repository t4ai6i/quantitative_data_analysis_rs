use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::stock_repository::data_format::csv::VecCSVFormat;
use crate::infrastructure::stock_repository::data_format::DataFormatType;
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use std::fs::read;
use std::path::PathBuf;

pub struct FileSystem {
    root: PathBuf,
}

impl FileSystem {
    pub fn new(path_buf: PathBuf) -> Self {
        Self { root: path_buf }
    }
}

#[async_trait]
impl StockRepository for FileSystem {
    ///
    /// # Examples
    /// ```ignore
    /// let repository = FileSystem::new(PathBuf::from("./assets/"));
    /// let code = "8473.T";
    /// let start_date = NaiveDate::default();
    /// let end_date = NaiveDate::default();
    /// let data_format_type = DataFormatType::CSVFormat { has_headers: true };
    /// let vec_stock =
    ///     repository.get_vec_stock(code.to_string(), start_date, end_date, data_format_type)?;
    /// assert_eq!(vec_stock.0.len(), 246);
    async fn get_vec_stock(
        &self,
        code: String,
        _: NaiveDate,
        _: NaiveDate,
        data_format_type: DataFormatType,
    ) -> Result<VecStock> {
        if let DataFormatType::CSVFormat { has_headers } = data_format_type {
            let filename = format!("{}.csv", code);
            let path = self.root.join(filename);
            let csv = read(path)?;
            let vec_stock = if has_headers {
                let vec_csv_format = VecCSVFormat::<true>::from(csv.as_slice());
                VecStock::from(vec_csv_format)
            } else {
                let vec_csv_format = VecCSVFormat::<false>::from(csv.as_slice());
                VecStock::from(vec_csv_format)
            };
            Ok(vec_stock)
        } else {
            bail!("Unsupported data format: {:?}", data_format_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::stock_repository::data_format::DataFormatType;
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
        let data_format_type = DataFormatType::CSVFormat { has_headers: true };
        let vec_stock = repository
            .get_vec_stock(code.to_string(), start_date, end_date, data_format_type)
            .await?;
        assert_eq!(vec_stock.0.len(), 246);
        Ok(())
    }
}
