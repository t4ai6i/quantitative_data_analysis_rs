use crate::domain::entity::buy_sell_signal::BuySellSignalType;
use crate::domain::entity::trend_reversal_analysis;
use crate::utils::custom_date_format;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

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

impl From<trend_reversal_analysis::TrendReversalAnalysis> for TrendReversalAnalysis {
    fn from(value: trend_reversal_analysis::TrendReversalAnalysis) -> Self {
        let trend_reversal_analysis::TrendReversalAnalysis {
            r#type,
            macos,
            ecp2,
            msesp,
            macps,
        } = value;
        Self {
            r#type,
            macos_date: macos.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
            ecp2_date: ecp2.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
            msesp_date: msesp.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
            macps_date: macps.map(|(date, _)| date).unwrap_or(NaiveDate::default()),
        }
    }
}
