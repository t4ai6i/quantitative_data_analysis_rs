use crate::domain::models::indicator::model::Indicator;
use crate::shared::float::validate_value;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub struct IndicatorAnalysisSet {
    pub indicator: Indicator,
}

/// 株価、財務指標などを元にした解析結果
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct IndicatorAnalysis {
    /// 終値日時
    pub close_date: NaiveDate,
    /// 開示日時
    pub disclosed_date: NaiveDate,
    /// Price Book-Value Ratio
    pub pbr: Option<f64>,
    /// Price Earnings Ratio
    pub per: Option<f64>,
    /// Mix Ratio
    pub mix: Option<f64>,
    /// Operating-Profit Ratio
    pub oppr: Option<f64>,
    /// Ordinary-Profit Ratio
    pub orpr: Option<f64>,
    /// Profit Ratio
    pub pr: Option<f64>,
}

impl From<IndicatorAnalysisSet> for IndicatorAnalysis {
    fn from(value: IndicatorAnalysisSet) -> Self {
        let IndicatorAnalysisSet { indicator } = value;
        let Indicator {
            close_date,
            disclosed_date,
            per,
            pbr,
            oppr,
            orpr,
            pr,
        } = indicator;

        let per = Some(per).and_then(validate_value);
        let pbr = Some(pbr).and_then(validate_value);
        /*
           MIX係数 = PBR * PER
           https://zaimani.com/financial-indicators/mix-coefficient/
           純資産と当期純利益の両方で株価の割安性を測定する指標。
           提唱者ベンジャミン・グレアム氏曰く、ミックス係数が22.5を下回る銘柄が割安である。
           さらに手堅く見るならば、ミックス係数が2を下回る銘柄が割安である。
        */
        let mix = per.zip(pbr).map(|(pbr, per)| pbr * per);

        /*
           営業利益率における適正水準の目安
           【標準的な水準】10%以下
           【優良水準】11%～20%
           【高水準だが、注意が必要】20%以上
           https://www.kaonavi.jp/dictionary/eigyoriekiritsu/
        */

        Self {
            close_date,
            disclosed_date,
            pbr,
            per,
            mix,
            orpr,
            oppr,
            pr,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::domain::models::indicator::model::Indicator;
    use crate::domain::models::indicator_analysis::model::{
        IndicatorAnalysis, IndicatorAnalysisSet,
    };

    #[test]
    fn indicator_analysis_test() {
        let indicator = Indicator {
            pbr: 1.0,
            per: 2.0,
            oppr: Some(50.0),
            orpr: Some(20.0),
            pr: Some(10.0),
            ..Default::default()
        };
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(1.0),
            per: Some(2.0),
            mix: Some(2.0),
            oppr: Some(50.0),
            orpr: Some(20.0),
            pr: Some(10.0),
            ..Default::default()
        };
        assert_eq!(actual, expected);
        let indicator = Indicator {
            pbr: 10.0,
            per: 20.0,
            ..Default::default()
        };

        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(10.0),
            per: Some(20.0),
            mix: Some(200.0),
            ..Default::default()
        };
        assert_eq!(actual, expected);

        let indicator = Indicator {
            pbr: 0.0,
            per: f64::NAN,
            oppr: None,
            orpr: None,
            pr: None,
            ..Default::default()
        };
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(0.0),
            per: None,
            mix: None,
            oppr: None,
            orpr: None,
            pr: None,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
