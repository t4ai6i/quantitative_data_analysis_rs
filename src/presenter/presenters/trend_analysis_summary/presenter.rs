use anyhow::Result;

use crate::presenter::presenters::trend_analysis_summary::{output, response};
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;

pub trait TrendAnalysisSummary {
    fn handle(
        &self,
        output: output::TrendAnalysisSummary,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysisSummary>;
}
