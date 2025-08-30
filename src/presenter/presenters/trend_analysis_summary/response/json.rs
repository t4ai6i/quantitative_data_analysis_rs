use anyhow::Result;

use crate::presenter::presenters::trend_analysis_summary::{output, presenter, response};
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::presenter::views::trend_analysis_summary::json::view;

pub struct JSON;

impl presenter::TrendAnalysisSummary for JSON {
    fn handle(
        &self,
        output: output::TrendAnalysisSummary,
        _: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysisSummary> {
        let view::JSON(rows) = view::JSON::from(output.trend_analyses);
        Ok(response::TrendAnalysisSummary::JSON { rows })
    }
}
