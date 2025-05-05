use rayon::prelude::*;
use std::backtrace::Backtrace;

use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use crate::infrastructure::repositories::company::data_format::csv::Csv;
use crate::infrastructure::repositories::company::data_format::tsv::Tsv;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;

#[async_trait]
impl repository::Company for FileSystem {
    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        todo!()
    }

    async fn get_company(&self, code: &str, _: &str) -> Result<model::Company> {
        let file = self.read_file().await?;
        let companies = if let DataFormat::JSON { .. } = self.data_format {
            serde_json::from_slice::<Vec<model::Company>>(&file)?
        } else {
            match self.data_format {
                DataFormat::CSV { has_headers, .. } => {
                    Csv::process_tabular_data(&file, has_headers)?
                }
                DataFormat::TSV { has_headers, .. } => {
                    Tsv::process_tabular_data(&file, has_headers)?
                }
                _ => bail!(
                    "Unsupported data format: {:?}\n{}",
                    self.data_format,
                    Backtrace::force_capture()
                ),
            }
        };
        companies
            .into_par_iter()
            .find_first(|company| company.code.eq(code))
            .with_context(|| {
                format!(
                    "Not found company: {}\n{}",
                    code,
                    Backtrace::force_capture()
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::company::model;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::repositories::company::file_system::FileSystem;
    use anyhow::Result;
    use std::path::PathBuf;

    #[tokio::test]
    async fn file_system_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let file_path = PathBuf::from("./assets/companies.json");
        let data_format = DataFormat::JSON { file_path };
        let repository = FileSystem::new(data_format);
        let actual = repository.get_company(code, market).await?;
        assert_eq!(
            actual,
            model::Company {
                code: "8473".to_string(),
                name: "ＳＢＩホールディングス".to_string(),
                market: "T".to_string(),
                symbol: "8473.T".to_string()
            }
        );

        let code = "1301";
        let market = "T";
        let file_path = PathBuf::from("./assets/companies.tsv");
        let data_format = DataFormat::TSV {
            has_headers: false,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let company = repository.get_company(code, market).await?;
        assert_eq!(
            company,
            model::Company {
                code: "1301".to_string(),
                name: "極洋".to_string(),
                market: "T".to_string(),
                symbol: "1301.T".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    #[should_panic]
    fn deserialize_invalid_json_test() {
        let json = br#" {"K": "#;
        let _ = serde_json::from_slice::<Vec<model::Company>>(json).unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn get_company_file_read_error_test() {
        let code = "8473";
        let market = "T";
        let file_path = PathBuf::from("./assets/not_exists.csv");
        let data_format = DataFormat::CSV {
            has_headers: true,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code, market).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn get_company_not_found_company_error_test() {
        let code = "8473";
        let market = "T";
        let file_path = PathBuf::from("./assets/companies.csv");
        let data_format = DataFormat::CSV {
            has_headers: true,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code, market).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn get_company_data_format_error_test() {
        let code = "8473";
        let market = "T";
        let data_format = DataFormat::YahooFinanceAPI;
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code, market).await.unwrap();
    }
}
