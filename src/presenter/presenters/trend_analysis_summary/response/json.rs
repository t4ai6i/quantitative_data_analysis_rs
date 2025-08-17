use anyhow::Result;

use crate::presenter::presenters::trend_analysis_summary::output;
use crate::presenter::presenters::trend_analysis_summary::presenter;
use crate::presenter::presenters::trend_analysis_summary::response;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::presenter::views::trend_analysis_summary::json::view;

pub struct JSON;

impl presenter::TrendAnalysisSummary for JSON {
    fn handle(
        &self,
        output: output::TrendAnalysisSummary,
        _: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysisSummary> {
        let view::JSON(vec_json_row) = view::JSON::from(output.trend_analyses);

        // let data = output.trend_analyses.vec_json();
        Ok(response::TrendAnalysisSummary::JSON { rows: vec_json_row })
    }
}
