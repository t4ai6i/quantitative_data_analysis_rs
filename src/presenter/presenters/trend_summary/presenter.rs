use crate::presenter::presenters::trend_summary::output;
use crate::presenter::presenters::trend_summary::response;
use crate::presenter::view_models::shared::crossover_pattern_filter::CrossoverPatternFilter;
use anyhow::Result;

pub trait TrendSummary {
    fn handle(
        &self,
        output: output::TrendSummary,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendSummary>;
}
