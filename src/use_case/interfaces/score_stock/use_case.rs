use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::score_stock::output;
use crate::use_case::interfaces::score_stock::input;

#[async_trait]
pub trait ScoreStock {
    async fn handle(&self, input: input::ScoreStock) -> Result<output::ScoreStock>;
}
