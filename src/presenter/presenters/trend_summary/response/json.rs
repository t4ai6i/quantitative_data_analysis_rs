use anyhow::Result;

use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::presenter::presenters::trend_summary::output;
use crate::presenter::presenters::trend_summary::presenter;
use crate::presenter::presenters::trend_summary::response;
use crate::presenter::view_models::trend_analysis::view_model::TrendAnalyses;

pub struct JSON;

impl presenter::TrendSummary for JSON {
    fn handle(
        &self,
        output: output::TrendSummary,
        _: MACOSPatternFilter,
    ) -> Result<response::TrendSummary> {
        let data = output.trend_analyses.vec_json();
        Ok(response::TrendSummary::JSON { data })
    }
}
