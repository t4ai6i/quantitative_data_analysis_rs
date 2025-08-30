use crate::presenter::views::trend_analysis_summary::json::view::JsonRows;

pub mod chart;
pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysisSummary {
    Chart { body: String },
    JSON { rows: JsonRows },
}
