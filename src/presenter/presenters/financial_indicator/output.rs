use crate::domain::models::indicator_analysis::model;

pub struct FinancialIndicator<const MIX_MIN: usize> {
    pub indicator_analysis: model::IndicatorAnalysis<MIX_MIN>,
}
