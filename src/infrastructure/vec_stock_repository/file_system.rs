use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::vec_stock_repository::data_format::csv::VecCSVFormat;
use crate::infrastructure::vec_stock_repository::data_format::DataFormatType;
use anyhow::{bail, Result};
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

impl StockRepository for FileSystem {
    fn get_vec_stock(
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
