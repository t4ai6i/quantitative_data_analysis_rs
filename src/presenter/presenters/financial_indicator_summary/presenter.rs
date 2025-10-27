use anyhow::Result;

use crate::presenter::presenters::financial_indicator_summary::{output, response};

pub trait FinancialIndicatorSummary {
    fn handle(
        &self,
        output: output::FinancialIndicatorSummary,
    ) -> Result<response::FinancialIndicatorSummary>;
}
