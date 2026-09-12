use crate::domain::models::company::model;
use crate::domain::repositories::company::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::company::structures::jquants_api::Response;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use query_string_builder::QueryStringOwned;
use rayon::prelude::*;
use reqwest::Client;
use serde_json::Value;

const EQUITIES_MASTER_URL: &str = "https://api.jquants.com/v2/equities/master";

#[async_trait]
impl repository::Company for JQuantsAPI {
    async fn get_row_company<'a>(
        &self,
        query: &queries::get_company::Query<'a>,
    ) -> Result<model::RowCompany> {
        let qs = QueryStringOwned::new().with("code", query.code);
        let url = format!("{EQUITIES_MASTER_URL}{qs}");
        let response = Client::new()
            .get(url)
            .header("x-api-key", self.api_key.to_string())
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let value = response["data"]
            .get(0)
            .with_context(|| format!("Not found company. code = {}", query.code))?;
        Ok(From::from(Response(value)))
    }

    async fn get_vec_row_company(&self) -> Result<Vec<model::RowCompany>> {
        let response = Client::new()
            .get(EQUITIES_MASTER_URL)
            .header("x-api-key", self.api_key.to_string())
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let Some(companies) = response["data"].as_array() else {
            bail!("[data] in response not found")
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
    use pretty_assertions::assert_eq;

    use crate::domain::models::company::model;
    use crate::domain::repositories::company::queries;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::shared::jquants_api::setup::Setup;

    #[tokio::test]
    async fn get_row_company_test() -> Result<()> {
        let token = Setup::run()?;
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
            product_category: Some("011".to_string()),
            symbol: Some("".to_string()),
        };
        assert_eq!(actual, expected);
        Ok(())
    }

    #[tokio::test]
    async fn get_vec_row_company_test() -> Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let vec_row_company = repository.get_vec_row_company().await?;
        assert!(vec_row_company.len() > 4400);
        Ok(())
    }
}
