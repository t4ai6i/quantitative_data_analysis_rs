use crate::presenter::presenters::financial_indicator::output;
use crate::use_case::interfaces::financial_indicator::input;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait FinancialIndicator {
    async fn handle<const MIX_MIN: usize>(
        &self,
        input: input::FinancialIndicator,
    ) -> Result<output::FinancialIndicator<MIX_MIN>>;
}
