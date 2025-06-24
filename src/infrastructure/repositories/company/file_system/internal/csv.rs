use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::file_system::FileSystem;
use crate::infrastructure::from_slice::FromSlice;
use crate::infrastructure::repositories::company::structures::internal::csv::Structure;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::backtrace::Backtrace;

pub struct Csv {
    pub has_headers: bool,
    pub file_system: FileSystem,
}

#[async_trait::async_trait]
impl repository::Company for Csv {
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
        Structure::process_tabular_data(&file, self.has_headers)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::company::model;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::file_system::FileSystem;
    use crate::infrastructure::repositories::company::file_system::internal::csv::Csv;
    use anyhow::Result;
    use std::path::PathBuf;

    #[tokio::test]
    async fn get_company_test() -> Result<()> {
        let code = "7647";
        let default_str = "";
        let file_path = PathBuf::from("./assets/companies.csv");
        let csv = Csv {
            has_headers: true,
            file_system: FileSystem::new(file_path),
        };
        let actual = csv.get_company(code, default_str).await?;
        let expected = model::Company {
            code: "7647".to_string(),
            name: "音通".to_string(),
            market: "東S".to_string(),
            symbol: "7647.T".to_string(),
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
