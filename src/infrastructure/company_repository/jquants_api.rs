use anyhow::Context;
use async_trait::async_trait;
use itertools::Itertools;
use query_string_builder::QueryString;
use reqwest::Client;

use crate::domain::entity::company::Company;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::infrastructure::jquants_api::JQuantsAPI;

const COMPANY_URL: &str = "https://api.jquants.com/v1/listed/info";

#[async_trait]
impl CompanyRepository for JQuantsAPI {
    async fn get_companies(&self) -> anyhow::Result<Vec<Company>> {
        let id_token = self.id_token.as_str();
        let response = Client::new()
            .get(COMPANY_URL)
            .bearer_auth(id_token)
            .send()
            .await?;
        let response = &response.json::<serde_json::Value>().await?;
        let company = response["info"]
            .as_array()
            .unwrap()
            .iter()
            .map(Self::company_from_value)
            .collect_vec();
        Ok(company)
    }

    async fn get_company(&self, code: &str, _market: &str) -> anyhow::Result<Company> {
        let qs = QueryString::dynamic().with_value("code", code);
        let company_url = format!("{COMPANY_URL}{qs}");
        let id_token = self.id_token.as_str();
        let response = Client::new()
            .get(company_url)
            .bearer_auth(id_token)
            .send()
            .await?;
        let response = &response.json::<serde_json::Value>().await?;
        let company = response["info"]
            .get(0)
            .with_context(|| format!("Not found company. {}", code))?;
        let company = Self::company_from_value(company);
        Ok(company)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::company::Company;
    use crate::domain::repository::company_repository::CompanyRepository;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use crate::utils::jquants_api::setup::Setup;
    use anyhow::Result;
    use rstest::*;

    #[fixture]
    async fn setup() -> Result<Token> {
        Setup::run().await
    }

    #[rstest]
    #[tokio::test]
    async fn get_company_test(#[future] setup: Result<Token>) -> Result<()> {
        let token = setup.await?;
        let code = "8473";
        let market = "T";
        let data_format = DataFormat::JQuantsAPI;
        let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;
        let company = repository.get_company(code, market).await?;
        assert_eq!(
            company,
            Company::new("84730", "SBI Holdings,Inc.", "0111", "")
        );
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_companies_test(#[future] setup: Result<Token>) -> Result<()> {
        let token = setup.await?;
        let data_format = DataFormat::JQuantsAPI;
        let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;
        let company = repository.get_companies().await;
        assert!(company.is_ok());
        Ok(())
    }
}
