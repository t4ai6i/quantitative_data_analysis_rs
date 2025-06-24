use rayon::prelude::*;

use crate::domain::models::stock::model::{Stock, Stocks};
use crate::domain::repositories::stock::repository;
use crate::infrastructure::symbol::Symbol;
use crate::infrastructure::yahoo_finance_api::{OffsetDateTimeWrapper, YahooFinanceAPI};
use crate::shared::tryhard::get_common_retry_future_config;
use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDate};
use num_traits::ToPrimitive;

#[async_trait]
impl repository::Stock for YahooFinanceAPI<'_> {
    async fn get_stock(
        &self,
        _code: &str,
        _market: &str,
        _target_date: NaiveDate,
    ) -> Result<Stock> {
        todo!()
    }

    async fn get_stocks(
        &self,
        code: &str,
        market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Stocks> {
        let symbol = Symbol::try_from((code, market))?;
        let start_date = OffsetDateTimeWrapper::from(start_date);
        let end_date = OffsetDateTimeWrapper::from(end_date);
        let retry_future_config = get_common_retry_future_config();
        let y_response = tryhard::retry_fn(|| {
            self.provider
                .get_quote_history(&symbol, start_date.0, end_date.0)
        });
        let y_response = y_response
            .with_config(retry_future_config)
            .await
            .with_context(|| {
                format!(
                    "Failed get_quote_history(). symbol: {}, start_date: {}, end_date: {}",
                    symbol.as_str(),
                    &start_date.0.to_string(),
                    &end_date.0.to_string()
                )
            })?;

        let vec_stock: Vec<Stock> = y_response
            .quotes()
            .with_context(|| format!("Failed fetching quotes: {}", symbol.as_str()))?
            .par_iter()
            .map(|quote| {
                let date = DateTime::from_timestamp(quote.timestamp.to_i64().unwrap_or(0), 0)
                    .unwrap()
                    .naive_utc();
                Stock {
                    date: date.date(),
                    open: quote.open,
                    high: quote.high,
                    low: quote.low,
                    close: quote.close,
                    adj_close: quote.adjclose,
                    volume: quote.volume,
                }
            })
            .collect();

        let mut stocks = Stocks::default();
        stocks.extend(vec_stock);
        Ok(stocks)
    }
}

/*
#[cfg(test)]
mod tests {
    // 通信が安定しないためテストを行わないようにした
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use chrono::NaiveDate;
    use yahoo_finance_api::YahooConnector;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let provider = YahooConnector::new()?;
        let repositories = YahooFinanceAPI::new(&provider);
        let stocks = repositories
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 244);
        let code = "V";
        let market = "";
        let repositories = YahooFinanceAPI::new(&provider);
        let stocks = repositories
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 251);
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_stock_code_not_found_test() {
        let code = "";
        let market = "";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let provider = YahooConnector::new().unwrap();
        let repositories = YahooFinanceAPI::new(&provider);
        let _ = repositories
            .get_stocks(code, market, start_date, end_date)
            .await
            .unwrap();
    }
}
*/
