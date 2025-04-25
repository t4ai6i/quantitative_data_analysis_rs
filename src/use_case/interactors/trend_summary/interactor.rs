use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::trend_summary::output;
use crate::use_case::interfaces::trend_summary::input;
use crate::use_case::interfaces::trend_summary::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendSummary;

#[async_trait]
impl use_case::TrendSummary for TrendSummary {
    async fn handle(&self, input: input::TrendSummary) -> Result<output::TrendSummary> {
        Ok(output::TrendSummary::new(input.trend_analyses))
    }
}
