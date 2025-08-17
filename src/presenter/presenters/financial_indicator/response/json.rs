use anyhow::Result;

use crate::presenter::presenters::financial_indicator::output;
use crate::presenter::presenters::financial_indicator::presenter;
use crate::presenter::presenters::financial_indicator::response;

pub struct Json;

impl presenter::FinancialIndicator for Json {
    fn handle(&self, output: output::FinancialIndicator) -> Result<response::FinancialIndicator> {
        Ok(response::FinancialIndicator::JSON {
            code: output.code,
            market: output.market,
            financial_indicator: output.financial_indicator,
        })
    }
}
