pub mod json;

use crate::domain::models::financial_indicator::model;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FinancialIndicator {
    JSON {
        code: String,
        market: String,
        financial_indicator: model::FinancialIndicator,
    },
}
