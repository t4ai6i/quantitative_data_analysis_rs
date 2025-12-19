use anyhow::{Context, Result, bail};
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
    async fn get_row_company<'a>(
        &self,
        query: &queries::get_company::Query<'a>,
    ) -> Result<model::RowCompany> {
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
        Ok(From::from(Response(value)))
    }

    async fn get_vec_row_company(&self) -> Result<Vec<model::RowCompany>> {
        let response = Client::new()
            .get(COMPANY_URL)
            .bearer_auth(self.id_token.clone())
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let Some(companies) = response["info"].as_array() else {
            bail!("[info] in response not found")
        };
        let vec_row_company = companies
            .par_iter()
            .map(|value| From::from(Response(value)))
            .collect();
        Ok(vec_row_company)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytestring::ByteString;
    use pretty_assertions::assert_eq;
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
    async fn get_row_company_test(#[future] setup: Result<ByteString>) -> Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_company::Query {
            code: "8473",
            ..Default::default()
        };
        let actual = repository.get_row_company(&query).await?;
        let expected = model::RowCompany {
            code: Some("84730".to_string()),
            name: Some("SBI Holdings,Inc.".to_string()),
            market: Some("0111".to_string()),
            symbol: Some("".to_string()),
        };
        assert_eq!(actual, expected);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_vec_row_company_test(#[future] setup: Result<ByteString>) -> Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let vec_row_company = repository.get_vec_row_company().await?;
        assert!(vec_row_company.len() > 4400);
        Ok(())
    }
}
