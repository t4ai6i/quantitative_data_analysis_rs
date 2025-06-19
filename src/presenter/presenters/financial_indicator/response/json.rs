use crate::presenter::presenters::financial_indicator::output;
use crate::presenter::presenters::financial_indicator::presenter;
use crate::presenter::presenters::financial_indicator::response;
use anyhow::Result;

pub struct JSON;

impl presenter::FinancialIndicator for JSON {
    fn handle(&self, output: output::FinancialIndicator) -> Result<response::FinancialIndicator> {
        Ok(response::FinancialIndicator::Json {
            code: output.code,
            indicator_analysis: output.indicator_analysis,
        })
    }
}
