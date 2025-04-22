use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::presenter::presenters::trend_summary::output;
use crate::presenter::presenters::trend_summary::response;
use anyhow::Result;

pub trait TrendSummary {
    fn handle(
        &self,
        output: output::TrendSummary,
        macos_pattern_filter: MACOSPatternFilter,
    ) -> Result<response::TrendSummary>;
}
