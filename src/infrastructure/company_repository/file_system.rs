use crate::domain::models::company::model::Company;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::infrastructure::company_repository::data_format::csv::Csv;
use crate::infrastructure::company_repository::data_format::tsv::Tsv;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use tokio::fs::read;

#[async_trait]
impl CompanyRepository for FileSystem {
    async fn get_companies(&self) -> Result<Vec<Company>> {
        todo!()
    }

    async fn get_company(&self, code: &str, _: &str) -> Result<Company> {
        let file_path = match self.data_format {
            DataFormat::JSON { ref file_path } => file_path,
            DataFormat::CSV { ref file_path, .. } => file_path,
            DataFormat::TSV { ref file_path, .. } => file_path,
            _ => {
                bail!(format!(
                    "Unsupported data format: {:?} at {}:{}",
                    self.data_format,
                    file!(),
                    line!()
                ));
            }
        };
        let file = read(file_path).await.with_context(|| {
            format!("File not found: {:?} at {}:{}", file_path, file!(), line!())
        })?;
        let companies = match self.data_format {
            DataFormat::JSON { .. } => {
                serde_json::from_slice::<Vec<Company>>(&file).with_context(|| {
                    format!(
                        "Invalid JSON: {:?} at {}:{}",
                        String::from_utf8_lossy(&file),
                        file!(),
                        line!()
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
        companies
            .into_iter()
            .find(|company: &Company| company.code.eq(code))
            .with_context(|| format!("Not found company: {} at {}:{}", code, file!(), line!()))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::company::model::Company;
    use crate::domain::repository::company_repository::CompanyRepository;
    use crate::infrastructure::company_repository::file_system::FileSystem;
    use crate::infrastructure::data_format::DataFormat;
    use anyhow::{Context, Result};
    use std::backtrace::Backtrace;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_vec_company_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let file_path = PathBuf::from("./assets/companies.json");
        let data_format = DataFormat::JSON { file_path };
        let repository = FileSystem::new(data_format);
        let actual = repository.get_company(code, market).await?;
        assert_eq!(
            actual,
            Company {
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
            Company {
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
    async fn get_vec_company_not_found_company_error_test() {
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
    async fn get_vec_company_data_format_error_test() {
        let code = "8473";
        let market = "T";
        let data_format = DataFormat::YahooFinanceAPI;
        let repository = FileSystem::new(data_format);
        let _ = repository.get_company(code, market).await.unwrap();
    }
}
