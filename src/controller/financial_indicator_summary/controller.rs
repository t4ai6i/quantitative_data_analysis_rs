use anyhow::Result;

use crate::presenter::presenters::financial_indicator::response::FinancialIndicators;
use crate::presenter::presenters::financial_indicator_summary::{presenter, response};
use crate::use_case::interfaces::financial_indicator_summary::{input, use_case};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FinancialIndicatorSummary<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> FinancialIndicatorSummary<'a, I, P>
where
    I: use_case::FinancialIndicatorSummary,
    P: presenter::FinancialIndicatorSummary,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        financial_indicators: FinancialIndicators,
    ) -> Result<response::FinancialIndicatorSummary> {
        let input = input::FinancialIndicatorSummary {
            financial_indicators,
        };
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
