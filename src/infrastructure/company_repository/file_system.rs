use crate::domain::entity::company::Company;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::infrastructure::company_repository::data_format::csv::CompanyCsvRow;
use crate::infrastructure::company_repository::data_format::DataFormat;
use crate::infrastructure::csv_ext::CsvExt;
use crate::infrastructure::file_system::FileSystem;
use anyhow::{bail, Result};
use async_trait::async_trait;
use itertools::Itertools;

#[async_trait]
impl CompanyRepository for FileSystem {
    async fn get_company(
        &self,
        code: impl Into<String> + Send,
        data_format: DataFormat,
    ) -> Result<Company> {
        let companies = if let DataFormat::Json = data_format {
            let path = self.root.join("companies.json");
            let json = tokio::fs::read(path).await?;
            serde_json::from_slice::<Vec<Company>>(&json)?
        } else if let DataFormat::CSV { has_headers } = data_format {
            let path = self.root.join("companies.csv");
            let csv = tokio::fs::read(path).await?;
            if has_headers {
                CompanyCsvRow::from_slice::<true>(csv.as_slice())
            } else {
                CompanyCsvRow::from_slice::<false>(csv.as_slice())
            }
        } else {
            bail!("Unsupported data format: {:?}", data_format);
        };
        let code = code.into().split('.').collect_vec()[0].to_string();
        let company = companies
            .into_iter()
            .find(|company: &Company| company.code.eq(&code))
            .unwrap_or(Company::default());
        Ok(company)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::company::Company;
    use crate::domain::repository::company_repository::CompanyRepository;
    use crate::infrastructure::company_repository::data_format::DataFormat;
    use crate::infrastructure::company_repository::file_system::FileSystem;
    use anyhow::Result;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_vec_company_test() -> Result<()> {
        let repository = FileSystem::new(PathBuf::from("./assets/"));
        let code = "8473.T";

        let data_format = DataFormat::CSV { has_headers: true };
        let company = repository.get_company(code, data_format).await?;
        assert_eq!(company, Company::default());

        let data_format = DataFormat::Json;
        let company = repository.get_company(code, data_format).await?;
        assert_eq!(
            company,
            Company {
                code: "8473".to_string(),
                name: "ＳＢＩホールディングス".to_string(),
                market: "東S".to_string(),
            }
        );
        Ok(())
    }
}
