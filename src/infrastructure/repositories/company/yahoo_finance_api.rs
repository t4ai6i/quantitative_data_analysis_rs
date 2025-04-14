use rayon::prelude::*;

use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use crate::utils::tryhard::get_common_retry_future_config;
use anyhow::{Context, Error, Result};
use async_trait::async_trait;
use yahoo_finance_api::YQuoteItem;

impl TryFrom<YQuoteItem> for model::Company {
    type Error = Error;

    fn try_from(value: YQuoteItem) -> std::result::Result<Self, Self::Error> {
        let code = value
            .symbol
            .split('.')
            .next()
            .with_context(|| format!("Cannot split quote.symbol: {}", value.symbol))?;
        Ok(model::Company {
            code: code.to_owned(),
            name: value.long_name,
            market: value.exchange,
            symbol: value.symbol,
        })
    }
}

#[async_trait]
impl<'a> repository::Company for YahooFinanceAPI<'a> {
    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        todo!()
    }

    async fn get_company(&self, code: &str, market: &str) -> Result<model::Company> {
        let name = model::Company::symbol(code, market);
        let retry_future_config = get_common_retry_future_config();
        let quotes = tryhard::retry_fn(|| self.provider.search_ticker(&name))
            .with_config(retry_future_config)
            .await
            .with_context(|| format!("Failed fetching code: {}", code))?
            .quotes;
        let quote = quotes
            .into_par_iter()
            .find_first(|quote| quote.symbol.eq(&name))
            .with_context(|| format!("Not found company: {}", &name))?;
        TryFrom::try_from(quote)
    }
}

/*
通信が安定しないためテストを行わないようにした
#[cfg(test)]
mod tests {
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use yahoo_finance_api::YahooConnector;
    use crate::domain::models::company::model;

    #[tokio::test]
    async fn get_company_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let data_format = DataFormat::YahooFinanceAPI;
        let provider = YahooConnector::new();
        let repositories = YahooFinanceAPI::new(&provider, data_format);
        let company = repositories.get_company(code, market).await?;
        assert_eq!(
            company,
            model::Company {
                code: "8473".to_string(),
                name: "SBI Holdings, Inc.".to_string(),
                market: "JPX".to_string(),
                symbol: "8473.T".to_string(),
            }
        );
        let code = "V";
        let market = "";
        let data_format = DataFormat::YahooFinanceAPI;
        let repositories = YahooFinanceAPI::new(&provider, data_format);
        let company = repositories.get_company(code, market).await?;
        assert_eq!(
            company,
            model::Company {
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
        let repositories = YahooFinanceAPI::new(&provider, data_format);
        let _ = repositories.get_company(code, market).await.unwrap();
    }
}
 */
