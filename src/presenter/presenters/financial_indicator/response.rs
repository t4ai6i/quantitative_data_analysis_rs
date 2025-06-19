mod json;

use crate::domain::models::indicator_analysis::model::IndicatorAnalysis;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FinancialIndicator {
    Json {
        code: String,
        indicator_analysis: IndicatorAnalysis, // Assuming this is a string representation of the analysis
    },
}
