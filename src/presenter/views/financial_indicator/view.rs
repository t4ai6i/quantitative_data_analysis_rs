use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::models::financial_indicator::model;
use crate::shared::custom_date_format;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct FinancialIndicator {
    /// 終値日時
    #[serde(with = "custom_date_format")]
    pub close_date: NaiveDate,
    /// 開示日時
    #[serde(with = "custom_date_format")]
    pub disclosed_date: NaiveDate,
    /// Price Book-Value Ratio/株価純資産倍率
    pub pbr: f64,
    /// Price Earnings Ratio/株価収益率
    pub per: f64,
    /// Operating-Profit Ratio/営業利益率
    pub oppr: Option<f64>,
    /// Ordinary-Profit Ratio/経常利益率
    pub orpr: Option<f64>,
    /// Profit Ratio/当期純利益率
    pub pr: Option<f64>,
    /// Mix Ratio/ミックス係数
    pub mix: Option<f64>,
}

impl From<&model::FinancialIndicator> for FinancialIndicator {
    fn from(value: &model::FinancialIndicator) -> Self {
        let model::FinancialIndicator {
            close_date,
            disclosed_date,
            pbr,
            per,
            oppr,
            orpr,
            pr,
            mix,
        } = value;
        Self {
            close_date: *close_date,
            disclosed_date: *disclosed_date,
            pbr: *pbr,
            per: *per,
            oppr: *oppr,
            orpr: *orpr,
            pr: *pr,
            mix: *mix,
        }
    }
}
