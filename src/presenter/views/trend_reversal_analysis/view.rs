use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::domain::models::trend_reversal_analysis::model;
use crate::shared::custom_date_format;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct TrendReversalAnalysis {
    pub r#type: BuySellSignalType,
    #[serde(with = "custom_date_format")]
    pub macos_date: NaiveDate,
    #[serde(with = "custom_date_format")]
    pub ecp2_date: NaiveDate,
    #[serde(with = "custom_date_format")]
    pub msesp_date: NaiveDate,
    #[serde(with = "custom_date_format")]
    pub macps_date: NaiveDate,
}

impl From<&model::TrendReversalAnalysis> for TrendReversalAnalysis {
    fn from(value: &model::TrendReversalAnalysis) -> Self {
        let model::TrendReversalAnalysis {
            r#type,
            macos,
            ecp2,
            msesp,
            macps,
        } = value;
        Self {
            r#type: *r#type,
            macos_date: macos.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
            ecp2_date: ecp2.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
            msesp_date: msesp.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
            macps_date: macps.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
        }
    }
}
