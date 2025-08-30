use crate::presenter::views::financial_indicator_summary::json::view::JsonRows;

pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FinancialIndicatorSummary {
    JSON { rows: JsonRows },
}
