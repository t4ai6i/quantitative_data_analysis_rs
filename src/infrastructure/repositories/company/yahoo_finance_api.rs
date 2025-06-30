use rayon::prelude::*;

use crate::domain::models::company::model;
use crate::domain::repositories::company::repository;
use crate::infrastructure::symbol::Symbol;
use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use crate::shared::tryhard::get_common_retry_future_config;
use anyhow::{Context, Result};
use async_trait::async_trait;

#[async_trait]
impl repository::Company for YahooFinanceAPI<'_> {
    async fn get_company(&self, code: &str, market: &str) -> Result<model::Company> {
        let symbol = Symbol::try_from((code, market))?;
        let retry_future_config = get_common_retry_future_config();
        let quotes = tryhard::retry_fn(|| self.provider.search_ticker(&symbol))
            .with_config(retry_future_config)
            .await
            .with_context(|| format!("Failed fetching code: {}", code))?
            .quotes;
        let quote = quotes
            .into_par_iter()
            .find_first(|quote| quote.symbol.eq(symbol.as_str()))
            .with_context(|| format!("Not found company: {}", symbol.as_str()))?;
        TryFrom::try_from(quote)
    }

    async fn get_companies(&self) -> Result<Vec<model::Company>> {
        todo!()
    }
}

/*
#[cfg(test)]
mod tests {
    // 通信が安定しないためテストを行わないようにした
    use crate::domain::models::company::model;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
    use anyhow::Result;
    use yahoo_finance_api::YahooConnector;

    #[tokio::test]
    async fn get_company_test() -> Result<()> {
        let code = "8473";
        let market = "T";
        let provider = YahooConnector::new()?;
        let repositories = YahooFinanceAPI::new(&provider);
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
        let repositories = YahooFinanceAPI::new(&provider);
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
        let provider = YahooConnector::new().unwrap();
        let repositories = YahooFinanceAPI::new(&provider);
        let _ = repositories.get_company(code, market).await.unwrap();
    }
}
*/
