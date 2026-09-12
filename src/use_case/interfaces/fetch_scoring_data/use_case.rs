use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::fetch_scoring_data::output;
use crate::use_case::interfaces::fetch_scoring_data::input;

#[async_trait]
pub trait FetchScoringData {
    async fn handle(&self, input: input::FetchScoringData) -> Result<output::FetchScoringData>;
}
