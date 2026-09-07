use crate::domain::models::stock::model;
use crate::domain::repositories::stock::queries::get_stocks_by_date::Query;
use crate::domain::repositories::stock::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::stock::structures::jquants_api::Response;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use chrono::NaiveDate;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;

const EQUITIES_BARS_DAILY_URL: &str = "https://api.jquants.com/v2/equities/bars/daily";

fn build_daily_bars_query(
    code: Option<&str>,
    date: Option<NaiveDate>,
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    pagination_key: Option<&str>,
) -> String {
    let mut qs = QueryString::dynamic();
    if let Some(code) = code {
        qs = qs.with_value("code", code);
    }
    if let Some(date) = date {
        qs = qs.with_value("date", date.to_string());
    }
    if let Some(from) = from {
        qs = qs.with_value("from", from.to_string());
    }
    if let Some(to) = to {
        qs = qs.with_value("to", to.to_string());
    }
    if let Some(pagination_key) = pagination_key {
        qs = qs.with_value("pagination_key", pagination_key);
    }
    qs.to_string()
}

#[async_trait]
impl repository::Stock for JQuantsAPI {
    async fn get_base_date_prices(&self, query: &Query) -> Result<model::BaseDatePrices> {
        self.fetch_base_date_prices(query.date).await
    }

    async fn get_row_stock<'a>(
        &self,
        query: &queries::get_stock::Query<'a>,
    ) -> Result<model::RowStock> {
        let qs = QueryString::dynamic()
            .with_value("code", query.code.unwrap_or(""))
            .with_value("from", query.target_date.to_string())
            .with_value("to", query.target_date.to_string());
        let daily_quotes_url = format!("{EQUITIES_BARS_DAILY_URL}{qs}");
        let response = Client::new()
            .get(daily_quotes_url)
            .header("x-api-key", self.api_key.to_string())
            .send()
            .await?;
        let response = response.json::<serde_json::Value>().await?;
        let value = response["data"]
            .get(0)
            .with_context(|| format!("Not found stock. code = {}", query.code.unwrap_or("")))?;
        Ok(From::from(Response(value)))
    }

    async fn get_vec_row_stock<'a>(
        &self,
        query: &queries::get_stocks::Query<'a>,
    ) -> Result<Vec<model::RowStock>> {
        let qs = QueryString::dynamic()
            .with_value("code", query.code.unwrap_or(""))
            .with_value("from", query.start_date.unwrap_or_default().to_string())
            .with_value("to", query.end_date.unwrap_or_default().to_string());
        let daily_quotes_url = format!("{EQUITIES_BARS_DAILY_URL}{qs}");
        let response = Client::new()
            .get(daily_quotes_url)
            .header("x-api-key", self.api_key.to_string())
            .send()
            .await?;
        let response = &mut response.json::<serde_json::Value>().await?;
        let vec_row_stock: Vec<model::RowStock> = response["data"]
            .as_array()
            .with_context(|| format!("data is empty. code = {}", query.code.unwrap_or("")))?
            .par_iter()
            .map(|value| From::from(Response(value)))
            .collect();
        Ok(vec_row_stock)
    }

    async fn get_vec_row_stock_by_date(&self, query: &Query) -> Result<Vec<model::RowStock>> {
        self.fetch_daily_bars(None, Some(query.date), None, None, None)
            .await
    }
}

impl JQuantsAPI {
    async fn fetch_base_date_prices(
        &self,
        effective_date: NaiveDate,
    ) -> Result<model::BaseDatePrices> {
        let mut rows = Vec::new();
        let mut pagination_key: Option<String> = None;

        loop {
            let qs = build_daily_bars_query(
                None,
                Some(effective_date),
                None,
                None,
                pagination_key.as_deref(),
            );
            let daily_quotes_url = format!("{EQUITIES_BARS_DAILY_URL}{qs}");
            let response = Client::new()
                .get(daily_quotes_url)
                .header("x-api-key", self.api_key.to_string())
                .send()
                .await?;
            let response = response.json::<serde_json::Value>().await?;
            let Some(page_rows) = response["data"].as_array() else {
                bail!(
                    "response[data] in response not found. effective_date = {}",
                    effective_date
                );
            };
            let page_rows = page_rows
                .iter()
                .map(|row| model::BaseDatePrice::try_from(Response(row)))
                .collect::<Result<Vec<_>>>()?;
            rows.extend(page_rows);
            pagination_key = response["pagination_key"]
                .as_str()
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            if pagination_key.is_none() {
                break;
            }
        }
        Ok(model::BaseDatePrices(rows))
    }

    async fn fetch_daily_bars(
        &self,
        code: Option<&str>,
        date: Option<NaiveDate>,
        from: Option<NaiveDate>,
        to: Option<NaiveDate>,
        target_date: Option<NaiveDate>,
    ) -> Result<Vec<model::RowStock>> {
        let mut rows = Vec::new();
        let mut pagination_key: Option<String> = None;

        loop {
            let qs = build_daily_bars_query(code, date, from, to, pagination_key.as_deref());
            let daily_quotes_url = format!("{EQUITIES_BARS_DAILY_URL}{qs}");
            let response = Client::new()
                .get(daily_quotes_url)
                .header("x-api-key", self.api_key.to_string())
                .send()
                .await?;
            let response = response.json::<serde_json::Value>().await?;
            let Some(page_rows) = response["data"].as_array() else {
                bail!(
                    "response[data] in response not found. code = {}, target_date = {}, from = {:?}, to = {:?}",
                    code.unwrap_or(""),
                    target_date
                        .or(date)
                        .map(|date| date.to_string())
                        .unwrap_or_default(),
                    from,
                    to
                );
            };

            let page_rows: Vec<model::RowStock> = page_rows
                .par_iter()
                .map(|value| From::from(Response(value)))
                .collect();
            rows.extend(page_rows);

            pagination_key = response["pagination_key"]
                .as_str()
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);
            if pagination_key.is_none() {
                break;
            }
        }

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::stock::model;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::shared::jquants_api::setup::Setup;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    #[tokio::test]
    async fn get_row_stock_test() -> anyhow::Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_stock::Query {
            code: Some("84730"),
            market: None,
            target_date: NaiveDate::from_ymd_opt(2025, 8, 27).unwrap(),
        };
        let actual = repository.get_row_stock(&query).await?;
        let expected = model::RowStock {
            date: NaiveDate::from_ymd_opt(2025, 8, 27),
            open: Some(6923.0),
            high: Some(6925.0),
            low: Some(6746.0),
            close: Some(6752.0),
            adj_close: Some(3376.0),
            volume: Some(3731200),
        };
        assert_eq!(actual, expected);
        Ok(())
    }

    #[tokio::test]
    async fn get_vec_row_stock_test() -> anyhow::Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_stocks::Query {
            code: Some("84730"),
            market: None,
            start_date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            end_date: Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap()),
        };
        let vec_row_stock = repository.get_vec_row_stock(&query).await?;
        assert_eq!(vec_row_stock.len(), 246);
        Ok(())
    }

    #[tokio::test]
    async fn get_base_date_prices_test() -> anyhow::Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_stocks_by_date::Query {
            date: NaiveDate::from_ymd_opt(2026, 8, 28).unwrap(),
        };

        let actual = repository.get_base_date_prices(&query).await?;

        assert!(!actual.is_empty());
        assert!(actual.iter().any(|row| !row.code.is_empty()));
        assert!(actual.iter().any(|row| row.adj_close.is_some()));

        Ok(())
    }
}
