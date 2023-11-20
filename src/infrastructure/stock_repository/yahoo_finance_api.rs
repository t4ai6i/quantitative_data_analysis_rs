use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::stock_repository::data_format::DataFormat;
use crate::infrastructure::yahoo_finance_api::{OffsetDateTimeWrapper, YahooFinanceAPI};
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
impl StockRepository for YahooFinanceAPI {
    async fn get_vec_stock(
        &self,
        code: impl Into<String> + Send,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format: DataFormat,
    ) -> Result<VecStock> {
        if let DataFormat::YahooFinanceAPI = data_format {
            let start_date = OffsetDateTimeWrapper::from(start_date);
            let end_date = OffsetDateTimeWrapper::from(end_date);
            let y_response = self
                .provider
                .get_quote_history(code.into().as_str(), start_date.0, end_date.0)
                .await?;
            let quotes = y_response.quotes()?;
            let vec_stock = VecStock::from(quotes);
            Ok(vec_stock)
        } else {
            bail!("Unsupported data format: {:?}", data_format);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::stock_repository::data_format::DataFormat;
    use crate::infrastructure::stock_repository::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let repository = YahooFinanceAPI::new();
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
}
