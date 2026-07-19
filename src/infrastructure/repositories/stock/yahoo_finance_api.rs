use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use rayon::prelude::*;

use crate::domain::models::stock::model;
use crate::domain::models::stock::model::RowStock;
use crate::domain::repositories::stock::queries::get_stocks_by_date::Query;
use crate::domain::repositories::stock::{queries, repository};
use crate::infrastructure::repositories::stock::structures::yahoo_finance_api::Response;
use crate::infrastructure::symbol::Symbol;
use crate::infrastructure::yahoo_finance_api::{OffsetDateTimeWrapper, YahooFinanceAPI};
use crate::shared::tryhard::get_common_retry_future_config;

#[async_trait]
impl repository::Stock for YahooFinanceAPI<'_> {
    async fn get_row_stock<'a>(
        &self,
        _: &queries::get_stock::Query<'a>,
    ) -> Result<model::RowStock> {
        todo!()
    }

    async fn get_vec_row_stock<'a>(
        &self,
        query: &queries::get_stocks::Query<'a>,
    ) -> Result<Vec<model::RowStock>> {
        let symbol = Symbol::try_from((query.code.unwrap_or(""), query.market.unwrap_or("")))?;
        let start_date = OffsetDateTimeWrapper::from(query.start_date.unwrap_or_default());
        let end_date = OffsetDateTimeWrapper::from(query.end_date.unwrap_or_default());
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

        let vec_row_stock: Vec<model::RowStock> = y_response
            .quotes()
            .with_context(|| format!("Failed fetching quotes: {}", symbol.as_str()))?
            .par_iter()
            .map(|quote| From::from(Response(quote)))
            .collect();
        Ok(vec_row_stock)
    }

    async fn get_vec_row_stock_by_date(&self, query: &Query) -> Result<Vec<RowStock>> {
        bail!(
            "get_vec_row_stock_by_date is not implemented for YahooFinanceAPI. Use get_vec_row_stock instead. date: {}",
            query.date
        );
    }
}

/*
#[cfg(test)]
mod tests {
    // 通信が安定しないためテストを行わないようにした
    use anyhow::Result;
    use chrono::NaiveDate;
    use yahoo_finance_api::YahooConnector;

    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;

    #[tokio::test]
    async fn get_vec_row_stock_test() -> Result<()> {
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();

        let provider = YahooConnector::new()?;
        let repository = YahooFinanceAPI::new(&provider);
        let query = queries::get_stocks::Query {
            code: Some("8473"),
            market: Some("T"),
            start_date: Some(start_date),
            end_date: Some(end_date),
        };
        let row_stocks = repository.get_vec_row_stock(&query).await?;
        assert_eq!(row_stocks.len(), 244);

        let query = queries::get_stocks::Query {
            code: Some("V"),
            market: None,
            start_date: Some(start_date),
            end_date: Some(end_date),
        };
        let row_stocks = repository.get_vec_row_stock(&query).await?;
        assert_eq!(row_stocks.len(), 251);
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_row_stock_code_not_found_test() {
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();

        let provider = YahooConnector::new().unwrap();
        let repository = YahooFinanceAPI::new(&provider);
        let query = queries::get_stocks::Query {
            code: Some(""),
            market: Some(""),
            start_date: Some(start_date),
            end_date: Some(end_date),
        };
        let _ = repository.get_vec_row_stock(&query).await.unwrap();
    }
}
*/
