use crate::presenter::views::trend_analysis_summary::json::view::JsonRow;

pub mod chart;
pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysisSummary {
    Chart { body: String },
    JSON { rows: Vec<JsonRow> },
}
