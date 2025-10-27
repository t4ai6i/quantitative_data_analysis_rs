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
    async fn get_row_company<'a>(
        &self,
        query: &queries::get_company::Query<'a>,
    ) -> Result<model::RowCompany> {
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
        Ok(From::from(quote))
    }

    async fn get_vec_row_company(&self) -> Result<Vec<model::RowCompany>> {
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
    async fn get_row_company_test() -> Result<()> {
        let provider = YahooConnector::new()?;
        let repository = YahooFinanceAPI::new(&provider);
        let query = queries::get_company::Query {
            code: "8473",
            market: Some("T"),
        };
        let row_company = repository.get_row_company(&query).await?;
        assert_eq!(
            row_company,
            model::RowCompany {
                code: Some("8473".to_string()),
                name: Some("SBI Holdings, Inc.".to_string()),
                market: Some("JPX".to_string()),
                symbol: Some("8473.T".to_string()),
            }
        );
        let repository = YahooFinanceAPI::new(&provider);
        let query = queries::get_company::Query {
            code: "V",
            ..Default::default()
        };
        let row_company = repository.get_row_company(&query).await?;
        assert_eq!(
            row_company,
            model::RowCompany {
                code: Some("V".to_string()),
                name: Some("Visa Inc.".to_string()),
                market: Some("NYQ".to_string()),
                symbol: Some("V".to_string()),
            }
        );
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn get_row_company_code_not_found_test() {
        let provider = YahooConnector::new().unwrap();
        let repository = YahooFinanceAPI::new(&provider);
        let query = queries::get_company::Query {
            code: "",
            ..Default::default()
        };
        let _ = repository.get_row_company(&query).await.unwrap();
    }
}
*/
