use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;
use serde_json::Value;

use crate::domain::models::company::model;
use crate::domain::repositories::company::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::company::structures::jquants_api::Response;

const COMPANY_URL: &str = "https://api.jquants.com/v1/listed/info";

#[async_trait]
impl repository::Company for JQuantsAPI {
    async fn get_company<'a>(
        &self,
        query: queries::get_company::Query<'a>,
    ) -> Result<model::Company> {
        let qs = QueryString::dynamic().with_value("code", query.code);
        let url = format!("{COMPANY_URL}{qs}");
        let response = Client::new()
            .get(url)
            .bearer_auth(self.id_token.clone())
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let value = response["info"]
            .get(0)
            .with_context(|| format!("Not found company. code = {}", query.code))?;
        TryFrom::try_from(Response(value))
    }

    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        let response = Client::new()
            .get(COMPANY_URL)
            .bearer_auth(self.id_token.clone())
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let Some(companies) = response["info"].as_array() else {
            bail!("[info] in response not found")
        };
        let company = companies
            .par_iter()
            .filter_map(|value| TryFrom::try_from(Response(value)).ok())
            .collect();
        Ok(company)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytestring::ByteString;
    use rstest::*;

    use crate::domain::models::company::model;
    use crate::domain::repositories::company::queries;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::shared::jquants_api::setup::Setup;

    #[fixture]
    async fn setup() -> Result<ByteString> {
        Setup::run().await
    }

    #[rstest]
    #[tokio::test]
    async fn get_company_test(#[future] setup: Result<ByteString>) -> Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_company::Query {
            code: "8473",
            ..Default::default()
        };
        let actual = repository.get_company(query).await?;
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
    async fn get_companies_test(#[future] setup: Result<ByteString>) -> Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let company = repository.get_companies().await;
        assert!(company.is_ok());
        Ok(())
    }
}
