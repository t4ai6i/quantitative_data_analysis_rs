use crate::domain::models::indicator::model::Indicator;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

pub struct IndicatorAnalysisSet {
    pub indicator: Indicator,
}

/// MIX係数などの指標を元にした解析結果
#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct IndicatorAnalysis {
    pub close_date: NaiveDate,
    pub disclosed_date: NaiveDate,
    pub pbr: Option<f64>,
    pub per: Option<f64>,
    pub mix: Option<f64>,
}

impl From<IndicatorAnalysisSet> for IndicatorAnalysis {
    fn from(value: IndicatorAnalysisSet) -> Self {
        let IndicatorAnalysisSet { indicator } = value;

        // NaN または無限大の値を検出する補助関数
        fn filter_invalid(value: f64) -> Option<f64> {
            if value.is_nan() || value.is_infinite() {
                None
            } else {
                Some(value)
            }
        }

        Self {
            close_date: indicator.close_date,
            disclosed_date: indicator.disclosed_date,
            pbr: filter_invalid(indicator.pbr),
            per: filter_invalid(indicator.per),
            mix: filter_invalid(indicator.mix),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::indicator::model::Indicator;
    use crate::domain::models::indicator_analysis::model::{
        IndicatorAnalysis, IndicatorAnalysisSet,
    };

    #[test]
    fn indicator_analysis_test() {
        let indicator = Indicator {
            per: 0.0,
            pbr: 0.0,
            mix: 0.0,
            ..Default::default()
        };
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            pbr: Some(0.0),
            per: Some(0.0),
            mix: Some(0.0),
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
