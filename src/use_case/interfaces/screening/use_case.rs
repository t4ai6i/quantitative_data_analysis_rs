use crate::domain::models::screening::model::ScreeningResults;
use crate::use_case::interfaces::screening::input;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ScreeningEngine {
    async fn handle(&self, input: input::Screening) -> Result<ScreeningResults>;
}
