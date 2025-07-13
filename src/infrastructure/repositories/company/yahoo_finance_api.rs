use rayon::prelude::*;

use crate::domain::models::company::model;
use crate::domain::repositories::company::{queries, repository};
use crate::infrastructure::symbol::Symbol;
use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;
use crate::shared::tryhard::get_common_retry_future_config;
use anyhow::{Context, Result};
use async_trait::async_trait;

#[async_trait]
impl repository::Company for YahooFinanceAPI<'_> {
    async fn get_company<'a>(
        &self,
        query: queries::get_company::Query<'a>,
    ) -> Result<model::Company> {
        let symbol = Symbol::try_from((query.code, query.market.unwrap_or("")))?;
        let retry_future_config = get_common_retry_future_config();
        let quotes = tryhard::retry_fn(|| self.provider.search_ticker(&symbol))
            .with_config(retry_future_config)
            .await
            .with_context(|| format!("Failed fetching code: {}", query.code))?
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
    use anyhow::Result;
    use yahoo_finance_api::YahooConnector;

    use crate::domain::models::company::model;
    use crate::domain::repositories::company::queries;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::yahoo_finance_api::YahooFinanceAPI;

    #[tokio::test]
    async fn get_company_test() -> Result<()> {
        let provider = YahooConnector::new()?;
        let repositories = YahooFinanceAPI::new(&provider);
        let query = queries::get_company::Query {
            code: "8473",
            market: Some("T"),
        };
        let company = repositories.get_company(query).await?;
        assert_eq!(
            company,
            model::Company {
                code: "8473".to_string(),
                name: "SBI Holdings, Inc.".to_string(),
                market: "JPX".to_string(),
                symbol: "8473.T".to_string(),
            }
        );
        let repositories = YahooFinanceAPI::new(&provider);
        let query = queries::get_company::Query {
            code: "V",
            ..Default::default()
        };
        let company = repositories.get_company(query).await?;
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
        let provider = YahooConnector::new().unwrap();
        let repositories = YahooFinanceAPI::new(&provider);
        let query = queries::get_company::Query {
            code: "",
            ..Default::default()
        };
        let _ = repositories.get_company(query).await.unwrap();
    }
}
*/
