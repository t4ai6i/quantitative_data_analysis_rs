use crate::presenter::presenters::financial_indicator::response::FinancialIndicators;

pub struct FinancialIndicatorSummary {
    pub financial_indicators: FinancialIndicators,
}

impl FinancialIndicatorSummary {
    pub fn new(financial_indicators: FinancialIndicators) -> Self {
        Self {
            financial_indicators,
        }
    }
}
