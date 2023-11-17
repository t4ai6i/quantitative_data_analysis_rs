use crate::domain::entity::stock::VecStock;
use crate::domain::repository::stock_repository::StockRepository;
use crate::infrastructure::stock_repository::data_format::yfapi_quote::VecQuote;
use crate::infrastructure::stock_repository::data_format::DataFormatType;
use anyhow::{bail, Result};
use async_trait::async_trait;
use chrono::NaiveDate;
use yahoo_finance_api::time::OffsetDateTime;
use yahoo_finance_api::YahooConnector;

pub struct YahooFinanceAPI;

impl YahooFinanceAPI {
    pub fn new() -> Self {
        Self {}
    }
}

struct OffsetDateTimeWrapper(pub(crate) OffsetDateTime);

impl From<NaiveDate> for OffsetDateTimeWrapper {
    fn from(value: NaiveDate) -> Self {
        let timestamp = value.and_hms_opt(0, 0, 0).unwrap().timestamp();
        let offset_date_time = OffsetDateTime::from_unix_timestamp(timestamp).unwrap();
        OffsetDateTimeWrapper(offset_date_time)
    }
}

#[async_trait]
impl StockRepository for YahooFinanceAPI {
    async fn get_vec_stock(
        &self,
        code: String,
        start_date: NaiveDate,
        end_date: NaiveDate,
        data_format_type: DataFormatType,
    ) -> Result<VecStock> {
        if let DataFormatType::YFAPIQuote = data_format_type {
            let provider = YahooConnector::new();
            let start_date = OffsetDateTimeWrapper::from(start_date);
            let end_date = OffsetDateTimeWrapper::from(end_date);
            let y_response = provider
                .get_quote_history(&code, start_date.0, end_date.0)
                .await?;
            let quotes = y_response.quotes()?;
            let quotes = VecQuote(quotes);
            let vec_stock = quotes.into();
            Ok(vec_stock)
        } else {
            bail!("Unsupported data format: {:?}", data_format_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repository::stock_repository::StockRepository;
    use crate::infrastructure::stock_repository::data_format::DataFormatType;
    use crate::infrastructure::stock_repository::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn get_vec_stock_test() -> Result<()> {
        let repository = YahooFinanceAPI::new();
        let code = "8473.T";
        let start_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2022, 12, 31).unwrap();
        let data_format_type = DataFormatType::YFAPIQuote;
        let vec_stock = repository
            .get_vec_stock(code.to_string(), start_date, end_date, data_format_type)
            .await?;
        assert_eq!(vec_stock.0.len(), 244);
        Ok(())
    }
}
