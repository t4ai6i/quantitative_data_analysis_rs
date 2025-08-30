use anyhow::Result;

use crate::presenter::presenters::financial_indicator_summary::{output, presenter, response};
use crate::presenter::views::financial_indicator_summary::json::view;

pub struct JSON;

impl presenter::FinancialIndicatorSummary for JSON {
    fn handle(
        &self,
        output: output::FinancialIndicatorSummary,
    ) -> Result<response::FinancialIndicatorSummary> {
        let view::JSON(rows) = view::JSON::from(output.financial_indicators);
        Ok(response::FinancialIndicatorSummary::JSON { rows })
    }
}
