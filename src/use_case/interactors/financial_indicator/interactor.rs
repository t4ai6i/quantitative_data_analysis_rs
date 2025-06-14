use crate::domain::models::indicator::model::Indicator;
use crate::domain::models::indicator_analysis::model::{IndicatorAnalysis, IndicatorAnalysisSet};
use crate::domain::repositories::{statement, stock};
use crate::presenter::presenters::financial_indicator::output;
use crate::use_case::interfaces::financial_indicator::{input, use_case};
use anyhow::Result;
use async_trait::async_trait;

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
    async fn handle<const MIX_MIN: usize>(
        &self,
        input: input::FinancialIndicator,
    ) -> Result<output::FinancialIndicator<MIX_MIN>> {
        let stock = self
            .stock_repository
            .get_stock(
                input.code.as_str(),
                input.market.as_str(),
                input.target_date,
            )
            .await?;

        let statement = self
            .statement_repository
            .get_statement(input.code.as_str())
            .await?;

        let indicator = Indicator::from((&stock, &statement));
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let indicator_analysis = IndicatorAnalysis::<MIX_MIN>::from(indicator_analysis_set);

        Ok(output::FinancialIndicator { indicator_analysis })
    }
}
