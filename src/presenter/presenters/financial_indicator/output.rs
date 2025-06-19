use crate::domain::models::indicator_analysis::model;

pub struct FinancialIndicator {
    pub code: String,
    pub indicator_analysis: model::IndicatorAnalysis,
}
