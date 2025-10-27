use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::financial_indicator_summary::output;
use crate::use_case::interfaces::financial_indicator_summary::input;
use crate::use_case::interfaces::financial_indicator_summary::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FinancialIndicatorSummary;

#[async_trait]
impl use_case::FinancialIndicatorSummary for FinancialIndicatorSummary {
    async fn handle(
        &self,
        input: input::FinancialIndicatorSummary,
    ) -> Result<output::FinancialIndicatorSummary> {
        Ok(output::FinancialIndicatorSummary::new(
            input.financial_indicators,
        ))
    }
}
