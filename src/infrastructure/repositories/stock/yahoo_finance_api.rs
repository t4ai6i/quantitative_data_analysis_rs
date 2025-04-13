use rayon::prelude::*;

use crate::domain::models::company::model;
use crate::domain::models::stock::model::{Stock, Stocks};
use crate::domain::repositories::stock::repository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::yahoo_finance_api::{OffsetDateTimeWrapper, YahooFinanceAPI};
use crate::utils::tryhard::get_common_retry_future_config;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime};

#[async_trait]
impl<'a> repository::Stock for YahooFinanceAPI<'a> {
    async fn get_stocks(
        &self,
        code: &str,
        market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Stocks> {
        if self.data_format.ne(&DataFormat::YahooFinanceAPI) {
            bail!(
                "Unsupported data format: {:?} at {}:{}",
                self.data_format,
                file!(),
                line!()
            );
        }
        let symbol = model::Company::symbol(code, market);
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
                    &symbol,
                    &start_date.0.to_string(),
                    &end_date.0.to_string()
                )
            })?;

        let vec_stock: Vec<Stock> = y_response
            .quotes()
            .with_context(|| format!("Failed fetching quotes: {}", &symbol))?
            .par_iter()
            .map(|quote| {
                let date =
                    NaiveDateTime::from_timestamp_opt(quote.timestamp as u32 as i64, 0).unwrap();
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
通信が安定しないためテストを行わないようにした
#[cfg(test)]
mod tests {
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::data_format::DataFormat;
    use anyhow::Result;
    use chrono::NaiveDate;
    use yahoo_finance_api::YahooConnector;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let data_format = DataFormat::YahooFinanceAPI;
        let provider = YahooConnector::new();
        let repositories = YahooFinanceAPI::new(&provider, data_format);
        let stocks = repositories
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 244);
        let code = "V";
        let market = "";
        let data_format = DataFormat::YahooFinanceAPI;
        let repositories = YahooFinanceAPI::new(&provider, data_format);
        let stocks = repositories
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 251);
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_stock_code_not_found_test() {
        let provider = YahooConnector::new();
        let code = "";
        let market = "";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let data_format = DataFormat::YahooFinanceAPI;
        let repositories = YahooFinanceAPI::new(&provider, data_format);
        let _ = repositories
            .get_stocks(code, market, start_date, end_date)
            .await
            .unwrap();
    }
}
 */
