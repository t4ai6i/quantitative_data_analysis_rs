use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;
use serde_json::Value;

use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::jquants_api::JQuantsAPI;

impl model::Company {
    fn new(value: &Value) -> Result<Self> {
        let code = value["Code"]
            .as_str()
            .with_context(|| "[Code] not found".to_string())?;
        let company_name_english = value["CompanyNameEnglish"]
            .as_str()
            .with_context(|| "[CompanyNameEnglish] not found".to_string())?;
        let market_code = value["MarketCode"]
            .as_str()
            .with_context(|| "[MarketCode] not found".to_string())?;
        Ok(Self {
            code: code.to_string(),
            name: company_name_english.to_string(),
            market: market_code.to_string(),
            symbol: "".to_string(),
        })
    }
}

const COMPANY_URL: &str = "https://api.jquants.com/v1/listed/info";

#[async_trait]
impl repository::Company for JQuantsAPI {
    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        let id_token = self.id_token.as_str();
        let response = Client::new()
            .get(COMPANY_URL)
            .bearer_auth(id_token)
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let Some(companies) = response["info"].as_array() else {
            bail!("[info] in response not found")
        };
        let company = companies
            .par_iter()
            .filter_map(|value| model::Company::new(value).ok())
            .collect();
        Ok(company)
    }

    async fn get_company(&self, code: &str, _market: &str) -> Result<model::Company> {
        let qs = QueryString::dynamic().with_value("code", code);
        let url = format!("{COMPANY_URL}{qs}");
        let id_token = self.id_token.as_str();
        let response = Client::new().get(url).bearer_auth(id_token).send().await?;
        let response = &response.json::<Value>().await?;
        let company = response["info"]
            .get(0)
            .with_context(|| format!("Not found company. {}", code))?;
        model::Company::new(company)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::*;

    use crate::domain::models::company::model;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use crate::utils::jquants_api::setup::Setup;

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
        let actual = repository.get_company(code, market).await?;
        let expected = model::Company {
            code: "84730".to_string(),
            name: "SBI Holdings,Inc.".to_string(),
            market: "0111".to_string(),
            symbol: "".to_string(),
        };
        assert_eq!(actual, expected);
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
