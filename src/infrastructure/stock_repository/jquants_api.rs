use anyhow::Context;
use async_trait::async_trait;
use chrono::NaiveDate;
use itertools::Itertools;
use num_traits::ToPrimitive;
use query_string_builder::QueryString;
use reqwest::Client;
use std::str::FromStr;

use crate::domain::entity::stock::{Stock, VecStock};
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::jquants_api::JQuantsAPI;

const DAILY_QUOTES_URL: &str = "https://api.jquants.com/v1/prices/daily_quotes";

#[async_trait]
impl StockRepository for JQuantsAPI {
    async fn get_vec_stock(
        &self,
        code: &str,
        _market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> anyhow::Result<VecStock> {
        let qs = QueryString::new()
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
        let stocks = response["daily_quotes"]
            .as_array()
            .with_context(|| format!("daily_quotes is empty. code = {}", code))?
            .iter()
            .map(|value| {
                let date = value["Date"].as_str().unwrap();
                let date = NaiveDate::from_str(date).unwrap();
                let open = value["Open"].as_f64().unwrap();
                let high = value["High"].as_f64().unwrap();
                let low = value["Low"].as_f64().unwrap();
                let close = value["Close"].as_f64().unwrap();
                let adj_close = value["AdjustmentClose"].as_f64().unwrap();
                let volume = value["Volume"].as_f64().unwrap().to_u64().unwrap();
                Stock {
                    date,
                    open,
                    high,
                    low,
                    close,
                    adj_close,
                    volume,
                }
            })
            .collect_vec();
        Ok(VecStock(stocks))
    }
}
#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::data_format::DataFormat;
    use chrono::NaiveDate;
    use rstest::*;

    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use crate::utils::jquants_api::setup::Setup;

    #[fixture]
    async fn setup() -> anyhow::Result<Token> {
        Setup::run().await
    }
    #[rstest]
    #[tokio::test]
    async fn get_vec_stock_test(#[future] setup: anyhow::Result<Token>) -> anyhow::Result<()> {
        let token = setup.await?;
        let code = "84730";
        let market = "";
        let start_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();
        let data_format = DataFormat::JQuantsAPI;
        let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;
        let vec_stock = repository
            .get_vec_stock(code, market, start_date, end_date)
            .await?;
        assert_eq!(vec_stock.0.len(), 246);
        Ok(())
    }
}
