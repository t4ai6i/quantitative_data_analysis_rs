use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::domain::models::trend_reversal_analysis::model;
use crate::shared::custom_serde::naive_date;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct TrendReversalAnalysis {
    pub r#type: BuySellSignalType,
    #[serde(with = "naive_date::primitive")]
    pub sma_cos_date: NaiveDate,
    #[serde(with = "naive_date::primitive")]
    pub sma_cps_date: NaiveDate,
    #[serde(with = "naive_date::primitive")]
    pub body_engulfing_date: NaiveDate,
    #[serde(with = "naive_date::primitive")]
    pub ms_es_date: NaiveDate,
}

impl From<&model::TrendReversalAnalysis> for TrendReversalAnalysis {
    fn from(value: &model::TrendReversalAnalysis) -> Self {
        let model::TrendReversalAnalysis {
            r#type,
            sma_cos,
            body_engulfing,
            ms_es,
            sma_cps,
        } = value;
        Self {
            r#type: *r#type,
            sma_cos_date: sma_cos
                .map(|(date, _)| date)
                .unwrap_or(NaiveDate::default()),
            sma_cps_date: sma_cps
                .map(|(date, _)| date)
                .unwrap_or(NaiveDate::default()),
            body_engulfing_date: body_engulfing
                .map(|(date, _)| date)
                .unwrap_or(NaiveDate::default()),
            ms_es_date: ms_es.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
        }
    }
}
