use crate::domain::entity::company::Company;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::infrastructure::company_repository::data_format::csv::Csv;
use crate::infrastructure::company_repository::data_format::tsv::Tsv;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use itertools::Itertools;
use std::backtrace::Backtrace;
use tokio::fs::read;

#[async_trait]
impl CompanyRepository for FileSystem {
    async fn get_company(&self, code: impl Into<String> + Send + Copy) -> Result<Company> {
        let file_path = match self.data_format {
            DataFormat::JSON { ref file_path } => file_path,
            DataFormat::CSV { ref file_path, .. } => file_path,
            DataFormat::TSV { ref file_path, .. } => file_path,
            _ => {
                bail!(format!(
                    "Unsupported data format: {:?}\n{}",
                    self.data_format,
                    Backtrace::force_capture()
                ));
            }
        };
        let file = read(file_path).await.with_context(|| {
            format!(
                "File not found: {:?}). \n{}",
                file_path,
                Backtrace::force_capture()
            )
        })?;
        let companies = match self.data_format {
            DataFormat::JSON { .. } => {
                serde_json::from_slice::<Vec<Company>>(&file).with_context(|| {
                    format!(
                        "Invalid JSON: {:?}). \n{}",
                        String::from_utf8_lossy(&file),
                        Backtrace::force_capture()
                    )
                })?
            }
            DataFormat::CSV { has_headers, .. } => {
                if has_headers {
                    Csv::from_slice::<true>(file.as_slice())
                } else {
                    Csv::from_slice::<false>(file.as_slice())
                }
            }
            DataFormat::TSV { has_headers, .. } => {
                if has_headers {
                    Tsv::from_slice::<true>(file.as_slice())
                } else {
                    Tsv::from_slice::<false>(file.as_slice())
                }
            }
            _ => vec![],
        };
        let just_code = code.into().split('.').collect_vec()[0].to_string();
        companies
            .into_iter()
            .find(|company: &Company| company.code.eq(&just_code))
            .with_context(|| {
                format!(
                    "Not found company: {}). \n{}",
                    code.into(),
                    Backtrace::force_capture()
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::company::Company;
    use crate::domain::repository::company_repository::CompanyRepository;
    use crate::infrastructure::company_repository::file_system::FileSystem;
    use crate::infrastructure::data_format::DataFormat;
    use anyhow::{Context, Result};
    use std::backtrace::Backtrace;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_vec_company_test() -> Result<()> {
        let code = "8473.T";
        let file_path = PathBuf::from("./assets/companies.json");
        let data_format = DataFormat::JSON { file_path };
        let repository = FileSystem::new(data_format);
        let company = repository.get_company(code).await?;
        assert_eq!(
            company,
            Company {
                code: "8473".to_string(),
                name: "ＳＢＩホールディングス".to_string(),
                market: "東S".to_string(),
            }
        );

        let code = "9984";
        let file_path = PathBuf::from("./assets/companies.tsv");
        let data_format = DataFormat::TSV {
            has_headers: false,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let company = repository.get_company(code).await?;
        assert_eq!(
            company,
            Company {
                code: "9984".to_string(),
                name: "ソフトバンクグループ".to_string(),
                market: "東証".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    #[should_panic]
    fn get_vec_company_invalid_json_test() {
        let json = br#" {"K": "#;
        let _ = serde_json::from_slice::<Vec<Company>>(json)
            .with_context(|| {
                format!(
                    "Invalid JSON: {:?}). \n{}",
                    String::from_utf8_lossy(json),
                    Backtrace::force_capture()
                )
            })
            .unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_company_file_read_error_test() {
        let code = "8473.T";
        let file_path = PathBuf::from("./assets/not_exists.csv");
        let data_format = DataFormat::CSV {
            has_headers: true,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_company_not_found_company_error_test() {
        let code = "8473.T";
        let file_path = PathBuf::from("./assets/companies.csv");
        let data_format = DataFormat::CSV {
            has_headers: true,
            file_path,
        };
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code).await.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_company_data_format_error_test() {
        let code = "8473.T";
        let data_format = DataFormat::YahooFinanceAPI;
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code).await.unwrap();
    }
}
