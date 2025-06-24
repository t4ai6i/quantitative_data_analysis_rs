use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::file_system::FileSystem;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::backtrace::Backtrace;

pub struct Json {
    pub file_system: FileSystem,
}

#[async_trait::async_trait]
impl repository::Company for Json {
    async fn get_company(&self, code: &str, _: &str) -> Result<model::Company> {
        let companies = self.get_companies().await?;
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

    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        let file = self.file_system.read_file().await?;
        let companies = serde_json::from_slice::<Vec<model::Company>>(&file)?;
        Ok(companies)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::company::model;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::file_system::FileSystem;
    use crate::infrastructure::repositories::company::file_system::internal::json::Json;
    use anyhow::Result;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_company_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let file_path = PathBuf::from("./assets/companies.json");
        let json = Json {
            file_system: FileSystem::new(file_path),
        };
        let actual = json.get_company(code, market).await?;
        let expected = model::Company {
            code: "8473".to_string(),
            name: "ＳＢＩホールディングス".to_string(),
            market: "T".to_string(),
            symbol: "8473.T".to_string(),
        };
        assert_eq!(actual, expected);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn deserialize_invalid_json_test() {
        let json = br#" {"K": "#;
        let _ = serde_json::from_slice::<Vec<model::Company>>(json).unwrap();
    }
}
