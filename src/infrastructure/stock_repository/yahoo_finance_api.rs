use crate::domain::models::company::model;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::yahoo_finance_api::{OffsetDateTimeWrapper, YahooFinanceAPI};
use crate::utils::tryhard::get_common_retry_future_config;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
impl<'a> StockRepository for YahooFinanceAPI<'a> {
    async fn get_stocks(
        &self,
        code: &str,
        market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Stocks> {
        if let DataFormat::YahooFinanceAPI = self.data_format {
            let symbol = model::Company::symbol(code, market);
            let start_date = OffsetDateTimeWrapper::from(start_date);
            let end_date = OffsetDateTimeWrapper::from(end_date);
            let retry_future_config = get_common_retry_future_config();
            let y_response = tryhard::retry_fn(|| {
                self.provider
                    .get_quote_history(&symbol, start_date.0, end_date.0)
            })
            .with_config(retry_future_config)
            .await
            .with_context(|| format!("Failed fetching symbol: {}", &symbol))?;
            let vec_stock = y_response
                .quotes()
                .map(Stocks::from)
                .with_context(|| format!("Failed mapping quotes into VecStock: {}", &symbol))?;
            Ok(vec_stock)
        } else {
            bail!(format!(
                "Unsupported data format: {:?} at {}:{}",
                self.data_format,
                file!(),
                line!()
            ));
        }
    }
}

/*
通信が安定しないためテストを行わないようにした
#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::stock_repository::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use chrono::NaiveDate;
    use yahoo_finance_api::YahooConnector;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let data_format = DataFormat::YahooFinanceAPI;
        let provider = YahooConnector::new();
        let repository = YahooFinanceAPI::new(&provider, data_format);
        let stocks = repository
            .get_stocks(code, market, start_date, end_date)
            .await?;
        assert_eq!(stocks.len(), 244);
        let code = "V";
        let market = "";
        let data_format = DataFormat::YahooFinanceAPI;
        let repository = YahooFinanceAPI::new(&provider, data_format);
        let stocks = repository
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
        let repository = YahooFinanceAPI::new(&provider, data_format);
        let _ = repository
            .get_stocks(code, market, start_date, end_date)
            .await
            .unwrap();
    }
}
*/
