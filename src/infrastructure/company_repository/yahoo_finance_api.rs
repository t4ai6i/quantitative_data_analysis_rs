use crate::domain::entity::company::Company;
use crate::domain::repository::company_repository::CompanyRepository;
use crate::infrastructure::data_format::DataFormat;
use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use std::backtrace::Backtrace;
use yahoo_finance_api::YQuoteItem;

#[async_trait]
impl<'a> CompanyRepository for YahooFinanceAPI<'a> {
    async fn get_company(
        &self,
        code: impl Into<String> + Send,
        data_format: DataFormat,
    ) -> Result<Company> {
        if let DataFormat::YahooFinanceAPI = data_format {
            let code = code.into();
            let y_search_result = self.provider.search_ticker(&code).await.with_context(|| {
                format!(
                    "Failed fetching code: {:?}). \n{}",
                    &code,
                    Backtrace::force_capture()
                )
            })?;
            let quotes = y_search_result.quotes;
            quotes
                .into_iter()
                .find(|quote| quote.symbol.eq(&code))
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
                data_format,
                Backtrace::force_capture()
            ));
        }
    }
}

impl From<YQuoteItem> for Company {
    fn from(value: YQuoteItem) -> Self {
        Company {
            code: value.symbol,
            name: value.short_name,
            market: value.exchange,
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
        let code = "8473.T";
        let provider = YahooConnector::new();
        let repository = YahooFinanceAPI::new(&provider);
        let data_format = DataFormat::YahooFinanceAPI;
        let company = repository.get_company(code, data_format).await?;
        assert_eq!(
            company,
            Company {
                code: "8473.T".to_string(),
                name: "SBI HOLDINGS INC".to_string(),
                market: "JPX".to_string(),
            }
        );
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_company_code_not_found_test() {
        let code = "";
        let provider = YahooConnector::new();
        let repository = YahooFinanceAPI::new(&provider);
        let data_format = DataFormat::YahooFinanceAPI;
        let _ = repository.get_company(code, data_format).await.unwrap();
    }
}
