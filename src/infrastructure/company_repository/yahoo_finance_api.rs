use crate::domain::entity::company::Company;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use crate::utils::tryhard::get_common_retry_future_config;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::backtrace::Backtrace;
use yahoo_finance_api::YQuoteItem;

#[async_trait]
impl<'a> CompanyRepository for YahooFinanceAPI<'a> {
    async fn get_company(&self, code: &str, market: &str) -> Result<Company> {
        if let DataFormat::YahooFinanceAPI = self.data_format {
            let name = Company::symbol(code, market);
            let retry_future_config = get_common_retry_future_config();
            let y_search_result = tryhard::retry_fn(|| self.provider.search_ticker(&name))
                .with_config(retry_future_config)
                .await
                .with_context(|| {
                    format!(
                        "Failed fetching code: {}. \n{}",
                        code,
                        Backtrace::force_capture()
                    )
                })?;
            let quotes = y_search_result.quotes;
            quotes
                .into_iter()
                .find(|quote| quote.symbol.eq(&name))
                .map(Company::from)
                .with_context(|| {
                    format!(
                        "Code fetching from yahoo! finance not exist: {:?}). \n{}",
                        &code,
                        Backtrace::force_capture()
                    )
                })
        } else {
            bail!(format!(
                "Unsupported data format: {:?}\n{}",
                self.data_format,
                Backtrace::force_capture()
            ));
        }
    }
}

impl From<YQuoteItem> for Company {
    fn from(value: YQuoteItem) -> Self {
        let mut split = value.symbol.split('.');
        let code = split
            .next()
            .with_context(|| format!("Unknown symbol: {}", value.symbol))
            .unwrap();
        Company {
            code: code.to_string(),
            name: value.long_name,
            market: value.exchange,
            symbol: value.symbol,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::company::Company;
    use crate::domain::repository::company_repository::CompanyRepository;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use yahoo_finance_api::YahooConnector;

    #[tokio::test]
    async fn get_company_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let data_format = DataFormat::YahooFinanceAPI;
        let provider = YahooConnector::new();
        let repository = YahooFinanceAPI::new(&provider, data_format);
        let company = repository.get_company(code, market).await?;
        assert_eq!(
            company,
            Company {
                code: "8473".to_string(),
                name: "SBI Holdings, Inc.".to_string(),
                market: "JPX".to_string(),
                symbol: "8473.T".to_string(),
            }
        );
        let code = "V";
        let market = "";
        let data_format = DataFormat::YahooFinanceAPI;
        let repository = YahooFinanceAPI::new(&provider, data_format);
        let company = repository.get_company(code, market).await?;
        assert_eq!(
            company,
            Company {
                code: "V".to_string(),
                name: "Visa Inc.".to_string(),
                market: "NYQ".to_string(),
                symbol: "V".to_string(),
            }
        );
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_company_code_not_found_test() {
        let code = "";
        let market = "";
        let provider = YahooConnector::new();
        let data_format = DataFormat::YahooFinanceAPI;
        let repository = YahooFinanceAPI::new(&provider, data_format);
        let _ = repository.get_company(code, market).await.unwrap();
    }
}
