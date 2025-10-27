use crate::domain::models::financial_indicator::model;

pub struct FinancialIndicator {
    pub code: String,
    pub market: String,
    pub financial_indicator: model::FinancialIndicator,
}
