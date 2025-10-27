use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::models::financial_indicator::model;
use crate::shared::custom_serde::{f64_nan, naive_date};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct FinancialIndicator {
    /// 終値日時
    #[serde(with = "naive_date::primitive")]
    pub close_date: NaiveDate,
    /// 開示日時
    #[serde(with = "naive_date::primitive")]
    pub disclosed_date: NaiveDate,
    /// Price Book-Value Ratio/株価純資産倍率
    #[serde(with = "f64_nan")]
    pub pbr: f64,
    /// Price Earnings Ratio/株価収益率
    #[serde(with = "f64_nan")]
    pub per: f64,
    /// Operating-Profit Ratio/営業利益率
    #[serde(with = "f64_nan")]
    pub oppr: f64,
    /// Ordinary-Profit Ratio/経常利益率
    #[serde(with = "f64_nan")]
    pub orpr: f64,
    /// Profit Ratio/当期純利益率
    #[serde(with = "f64_nan")]
    pub pr: f64,
    /// Mix Ratio/ミックス係数
    #[serde(with = "f64_nan")]
    pub mix: f64,
    /// Return on Equity/自己資本利益率
    #[serde(with = "f64_nan")]
    pub roe: f64,
    /// Return on Assets/総資産利益率
    #[serde(with = "f64_nan")]
    pub roa: f64,
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
            roe,
            roa,
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
            roe: *roe,
            roa: *roa,
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::presenter::views::financial_indicator::view::FinancialIndicator;

    #[test]
    fn float_nan_test() {
        let financial_indicator = FinancialIndicator {
            close_date: Default::default(),
            disclosed_date: Default::default(),
            pbr: 0.0,
            per: 0.0,
            oppr: 0.0,
            orpr: 0.0,
            pr: 0.0,
            mix: f64::NAN,
            roe: 0.0,
            roa: 0.0,
        };

        let actual = serde_json::to_string_pretty(&financial_indicator).unwrap();
        let expected = indoc! {r#"
           {
             "close_date": "1970-01-01",
             "disclosed_date": "1970-01-01",
             "pbr": 0.0,
             "per": 0.0,
             "oppr": 0.0,
             "orpr": 0.0,
             "pr": 0.0,
             "mix": null,
             "roe": 0.0,
             "roa": 0.0
           }"#};
        assert_eq!(actual, expected);

        let actual = serde_json::from_str::<FinancialIndicator>(expected).unwrap();
        assert_eq!(actual.close_date, financial_indicator.close_date);
        assert_eq!(actual.disclosed_date, financial_indicator.disclosed_date);
        assert_eq!(actual.pbr, financial_indicator.pbr);
        assert_eq!(actual.per, financial_indicator.per);
        assert_eq!(actual.oppr, financial_indicator.oppr);
        assert_eq!(actual.orpr, financial_indicator.orpr);
        assert_eq!(actual.pr, financial_indicator.pr);
        assert!(actual.mix.is_nan());
        assert_eq!(actual.roe, financial_indicator.roe);
        assert_eq!(actual.roa, financial_indicator.roa);
    }
}
