use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDate;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;

use crate::domain::models::stock::model;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock::repository;
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::stock::structures::jquants_api::Response;

const DAILY_QUOTES_URL: &str = "https://api.jquants.com/v1/prices/daily_quotes";

#[async_trait]
impl repository::Stock for JQuantsAPI {
    async fn get_stock(
        &self,
        _code: &str,
        _market: &str,
        _target_date: NaiveDate,
    ) -> anyhow::Result<model::Stock> {
        todo!()
    }

    async fn get_stocks(
        &self,
        code: &str,
        _market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> anyhow::Result<Stocks> {
        let qs = QueryString::dynamic()
            .with_value("code", code)
            .with_value("from", start_date.to_string())
            .with_value("to", end_date.to_string());
        let daily_quotes_url = format!("{DAILY_QUOTES_URL}{qs}");
        let id_token = self.id_token.as_str();
        let response = Client::new()
            .get(daily_quotes_url)
            .bearer_auth(id_token)
            .send()
            .await?;
        let response = &mut response.json::<serde_json::Value>().await?;
        let vec_stock: Vec<model::Stock> = response["daily_quotes"]
            .as_array()
            .with_context(|| format!("daily_quotes is empty. code = {}", code))?
            .par_iter()
            .filter_map(|value| TryFrom::try_from(Response(value)).ok())
            .collect();
        let mut stocks = Stocks::default();
        stocks.extend(vec_stock);
        Ok(stocks)
    }
}
#[cfg(test)]
mod tests {
    use crate::domain::repositories::stock::repository::Stock;
    use chrono::NaiveDate;
    use rstest::*;

    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use crate::shared::jquants_api::setup::Setup;

    #[fixture]
    async fn setup() -> anyhow::Result<Token> {
        Setup::run().await
    }
    #[rstest]
    #[tokio::test]
    async fn get_stocks_test(#[future] setup: anyhow::Result<Token>) -> anyhow::Result<()> {
        let token = setup.await?;
        let code = "84730";
        let market = "";
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        let repository = JQuantsAPI::new(token.id_token.value)?;
        let stocks = repository
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 246);
        Ok(())
    }
}
