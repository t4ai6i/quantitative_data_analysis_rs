use deref_derive::{Deref, DerefMut};

use crate::domain::models::financial_indicator::model;

pub mod json;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FinancialIndicator {
    JSON {
        code: String,
        market: String,
        financial_indicator: model::FinancialIndicator,
    },
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct FinancialIndicators(pub Vec<FinancialIndicator>);
