use anyhow::Result;
use async_trait::async_trait;
use chrono::Days;

use crate::domain::models::financial_indicator::model;
use crate::domain::repositories::statement::queries::get_statement;
use crate::domain::repositories::stock::queries::get_stocks;
use crate::domain::repositories::{statement, stock};
use crate::presenter::presenters::financial_indicator::output;
use crate::use_case::interfaces::financial_indicator::{input, use_case};

const DAYS_7: u64 = 7;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FinancialIndicator<'a, SR, SMR> {
    stock_repository: &'a SR,
    statement_repository: &'a SMR,
}

impl<'a, SR, SMR> FinancialIndicator<'a, SR, SMR> {
    pub fn new(stock_repository: &'a SR, statement_repository: &'a SMR) -> Self {
        Self {
            stock_repository,
            statement_repository,
        }
    }
}

#[async_trait]
impl<SR, SMR> use_case::FinancialIndicator for FinancialIndicator<'_, SR, SMR>
where
    SR: stock::repository::Stock + Send + Sync,
    SMR: statement::repository::Statement + Send + Sync,
{
    async fn handle(&self, input: input::FinancialIndicator) -> Result<output::FinancialIndicator> {
        let start_date = input.target_date.checked_sub_days(Days::new(DAYS_7));
        let query = get_stocks::Query {
            code: Some(input.code.as_str()),
            market: Some(input.market.as_str()),
            start_date,
            end_date: Some(input.target_date),
        };
        let stocks = self.stock_repository.get_stocks(&query).await?;
        let stock = stocks.last().ok_or_else(|| {
            anyhow::anyhow!("no valid stock within one-week period. {:?}", &query)
        })?;

        let query = get_statement::Query {
            code: input.code.as_str(),
        };
        let statement = self.statement_repository.get_statement(&query).await?;

        let financial_indicator = model::FinancialIndicator::from((stock, &statement));

        Ok(output::FinancialIndicator {
            code: input.code,
            market: input.market,
            financial_indicator,
        })
    }
}
