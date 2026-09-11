use crate::presenter::presenters::technical_analysis::output;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait TechnicalAnalysis {
    async fn handle(
        &self,
        input: super::input::TechnicalAnalysis,
    ) -> Result<output::TechnicalAnalysis>;
}
