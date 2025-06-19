use crate::presenter::presenters::financial_indicator::output;
use crate::presenter::presenters::financial_indicator::response;
use anyhow::Result;
pub trait FinancialIndicator {
    fn handle(&self, output: output::FinancialIndicator) -> Result<response::FinancialIndicator>;
}
