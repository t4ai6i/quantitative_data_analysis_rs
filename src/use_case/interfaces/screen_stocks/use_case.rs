use crate::presenter::presenters::screen_stocks::output;
use crate::use_case::interfaces::screen_stocks::input;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ScreenStocks {
    async fn handle(&self, input: input::ScreenStocks) -> Result<output::ScreenStocks>;
}
