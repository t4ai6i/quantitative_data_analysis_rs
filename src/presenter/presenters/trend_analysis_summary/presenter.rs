use anyhow::Result;

use crate::presenter::presenters::trend_analysis_summary::output;
use crate::presenter::presenters::trend_analysis_summary::response;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;

pub trait TrendAnalysisSummary {
    fn handle(
        &self,
        output: output::TrendAnalysisSummary,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysisSummary>;
}
