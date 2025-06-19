use crate::presenter::presenters::financial_indicator::presenter;
use crate::presenter::presenters::financial_indicator::response;
use crate::use_case::interfaces::financial_indicator::input;
use crate::use_case::interfaces::financial_indicator::use_case;
use anyhow::Result;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FinancialIndicator<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> FinancialIndicator<'a, I, P>
where
    I: use_case::FinancialIndicator,
    P: presenter::FinancialIndicator,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        code: impl Into<String>,
        market: impl Into<String>,
        target_date: chrono::NaiveDate,
    ) -> Result<response::FinancialIndicator> {
        let input = input::FinancialIndicator {
            code: code.into(),
            market: market.into(),
            target_date,
        };
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
