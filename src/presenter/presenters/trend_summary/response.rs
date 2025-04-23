use crate::presenter::view_models::analysis::Analysis;

pub mod chart;
pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendSummary {
    Chart { body: String },
    JSON { data: Vec<Analysis> },
}
