use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::financial_indicator_summary::output;
use crate::use_case::interfaces::financial_indicator_summary::input;

#[async_trait]
pub trait FinancialIndicatorSummary {
    async fn handle(
        &self,
        input: input::FinancialIndicatorSummary,
    ) -> Result<output::FinancialIndicatorSummary>;
}
