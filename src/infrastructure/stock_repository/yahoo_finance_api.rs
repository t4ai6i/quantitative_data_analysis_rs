use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::yahoo_finance_api::{OffsetDateTimeWrapper, YahooFinanceAPI};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use std::backtrace::Backtrace;

#[async_trait]
impl<'a> StockRepository for YahooFinanceAPI<'a> {
    async fn get_vec_stock(
        &self,
        code: impl Into<String> + Send,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format: DataFormat,
    ) -> Result<VecStock> {
        if let DataFormat::YahooFinanceAPI = data_format {
            let code = code.into();
            let start_date = OffsetDateTimeWrapper::from(start_date);
            let end_date = OffsetDateTimeWrapper::from(end_date);
            let y_response = self
                .provider
                .get_quote_history(&code, start_date.0, end_date.0)
                .await
                .with_context(|| {
                    format!(
                        "Failed fetching code: {:?}). \n{}",
                        &code,
                        Backtrace::force_capture()
                    )
                })?;
            let vec_stock = y_response.quotes().map(VecStock::from).with_context(|| {
                format!(
                    "Failed mapping quotes into VecStock: {:?}). \n{}",
                    &code,
                    Backtrace::force_capture()
                )
            })?;
            Ok(vec_stock)
        } else {
            bail!(format!(
                "Unsupported data format: {:?}\n{}",
                data_format,
                Backtrace::force_capture()
            ));
        }
    }
}

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
        let provider = YahooConnector::new();
        let repository = YahooFinanceAPI::new(&provider);
        let code = "8473.T";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let data_format = DataFormat::YahooFinanceAPI;
        let vec_stock = repository
            .get_vec_stock(code, start_date, end_date, data_format)
            .await?;
        assert_eq!(vec_stock.0.len(), 244);
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_vec_stock_code_not_found_test() {
        let provider = YahooConnector::new();
        let repository = YahooFinanceAPI::new(&provider);
        let code = "";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let data_format = DataFormat::YahooFinanceAPI;
        let _ = repository
            .get_vec_stock(code, start_date, end_date, data_format)
            .await
            .unwrap();
    }
}
