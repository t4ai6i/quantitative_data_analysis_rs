use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::indicator::model::Indicator;
use crate::domain::models::indicator_analysis::model::{IndicatorAnalysis, IndicatorAnalysisSet};
use crate::domain::repositories::statement::queries::get_statement;
use crate::domain::repositories::stock::queries::get_stock;
use crate::domain::repositories::{statement, stock};
use crate::presenter::presenters::financial_indicator::output;
use crate::use_case::interfaces::financial_indicator::{input, use_case};

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
    SR: stock::repository::Stock + Sync,
    SMR: statement::repository::Statement + Sync,
{
    async fn handle(&self, input: input::FinancialIndicator) -> Result<output::FinancialIndicator> {
        let query = get_stock::Query {
            code: Some(input.code.as_str()),
            market: Some(input.market.as_str()),
            target_date: input.target_date,
        };
        let stock = self.stock_repository.get_stock(query).await?;

        let query = get_statement::Query {
            code: input.code.as_str(),
        };
        let statement = self.statement_repository.get_statement(query).await?;

        let indicator = Indicator::from((&stock, &statement));
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let indicator_analysis = IndicatorAnalysis::from(indicator_analysis_set);

        Ok(output::FinancialIndicator {
            code: input.code,
            indicator_analysis,
        })
    }
}
