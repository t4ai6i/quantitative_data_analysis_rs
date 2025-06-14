use crate::domain::models::indicator::model::Indicator;
use crate::shared::float::validate_value;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub struct IndicatorAnalysisSet {
    pub indicator: Indicator,
}

/// MIX係数などの指標を元にした解析結果
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct IndicatorAnalysis<const MIX_MIN: usize> {
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
}

impl<const MIX_MIN: usize> From<IndicatorAnalysisSet> for IndicatorAnalysis<MIX_MIN> {
    fn from(value: IndicatorAnalysisSet) -> Self {
        let IndicatorAnalysisSet { indicator } = value;
        let Indicator {
            close_date,
            disclosed_date,
            per,
            pbr,
        } = indicator;

        let per = Some(per).and_then(validate_value);
        let pbr = Some(pbr).and_then(validate_value);
        // MIX = PBR * PER
        // https://zaimani.com/financial-indicators/mix-coefficient/
        let mix = per.zip(pbr).and_then(|(pbr, per)| {
            let mix = pbr * per;
            // 純資産と当期純利益の両方で株価の割安性を測定する指標。
            // 提唱者ベンジャミン・グレアム氏曰く、ミックス係数が22.5を下回る銘柄が割安である。
            // さらに手堅く見るならば、ミックス係数が2を下回る銘柄が割安である。
            if mix <= MIX_MIN as f64 {
                Some(mix)
            } else {
                None
            }
        });

        Self {
            close_date,
            disclosed_date,
            pbr,
            per,
            mix,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::indicator::model::Indicator;
    use crate::domain::models::indicator_analysis::model::{
        IndicatorAnalysis, IndicatorAnalysisSet,
    };

    const MIX_MIN: usize = 2;

    #[test]
    fn indicator_analysis_test() {
        let indicator = Indicator {
            pbr: 1.0,
            per: 2.0,
            ..Default::default()
        };
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::<MIX_MIN>::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(1.0),
            per: Some(2.0),
            mix: Some(2.0),
            ..Default::default()
        };
        assert_eq!(actual, expected);
        let indicator = Indicator {
            pbr: 10.0,
            per: 20.0,
            ..Default::default()
        };

        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::<MIX_MIN>::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(10.0),
            per: Some(20.0),
            mix: None,
            ..Default::default()
        };
        assert_eq!(actual, expected);

        let indicator = Indicator {
            pbr: 0.0,
            per: f64::NAN,
            ..Default::default()
        };
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::<MIX_MIN>::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(0.0),
            per: None,
            mix: None,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
