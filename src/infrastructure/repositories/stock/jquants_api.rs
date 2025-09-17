use anyhow::Context;
use async_trait::async_trait;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;

use crate::domain::models::stock::model;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::stock::structures::jquants_api::Response;

const DAILY_QUOTES_URL: &str = "https://api.jquants.com/v1/prices/daily_quotes";

#[async_trait]
impl repository::Stock for JQuantsAPI {
    async fn get_stock<'a>(
        &self,
        query: queries::get_stock::Query<'a>,
    ) -> anyhow::Result<model::Stock> {
        let qs = QueryString::dynamic()
            .with_value("code", query.code.unwrap_or(""))
            .with_value("from", query.target_date.to_string())
            .with_value("to", query.target_date.to_string());
        let daily_quotes_url = format!("{DAILY_QUOTES_URL}{qs}");
        let response = Client::new()
            .get(daily_quotes_url)
            .bearer_auth(self.id_token.clone())
            .send()
            .await?;
        let response = response.json::<serde_json::Value>().await?;
        let value = response["daily_quotes"]
            .get(0)
            .with_context(|| format!("Not found stock. code = {}", query.code.unwrap_or("")))?;
        TryFrom::try_from(Response(value))
    }

    async fn get_stocks<'a>(
        &self,
        query: queries::get_stocks::Query<'a>,
    ) -> anyhow::Result<Stocks> {
        let qs = QueryString::dynamic()
            .with_value("code", query.code.unwrap_or(""))
            .with_value("from", query.start_date.unwrap_or_default().to_string())
            .with_value("to", query.end_date.unwrap_or_default().to_string());
        let daily_quotes_url = format!("{DAILY_QUOTES_URL}{qs}");
        let response = Client::new()
            .get(daily_quotes_url)
            .bearer_auth(self.id_token.clone())
            .send()
            .await?;
        let response = &mut response.json::<serde_json::Value>().await?;
        let vec_stock: Vec<model::Stock> = response["daily_quotes"]
            .as_array()
            .with_context(|| format!("daily_quotes is empty. code = {}", query.code.unwrap_or("")))?
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
    use bytestring::ByteString;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use rstest::*;

    use crate::domain::models::stock::model;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::shared::jquants_api::setup::Setup;

    #[fixture]
    async fn setup() -> anyhow::Result<ByteString> {
        Setup::run().await
    }

    #[rstest]
    #[tokio::test]
    async fn get_stock_test(#[future] setup: anyhow::Result<ByteString>) -> anyhow::Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_stock::Query {
            code: Some("84730"),
            market: None,
            target_date: NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        };
        let actual = repository.get_stock(query).await?;
        let expected = model::Stock {
            date: NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
            open: 6923.0,
            high: 6925.0,
            low: 6746.0,
            close: 6752.0,
            adj_close: 6752.0,
            volume: 3731200,
        };
        assert_eq!(actual, expected);
        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn get_stocks_test(#[future] setup: anyhow::Result<ByteString>) -> anyhow::Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_stocks::Query {
            code: Some("84730"),
            market: None,
            start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        };
        let stocks = repository.get_stocks(query).await?;
        assert_eq!(stocks.len(), 246);
        Ok(())
    }
}
