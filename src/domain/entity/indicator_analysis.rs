use crate::domain::entity::indicator::Indicator;
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
    pub mix: Option<f64>,
}

impl From<IndicatorAnalysisSet> for IndicatorAnalysis {
    fn from(value: IndicatorAnalysisSet) -> Self {
        let IndicatorAnalysisSet { indicator } = value;
        let mix = if indicator.mix.is_nan() {
            None
        } else {
            Some(indicator.mix)
        };
        Self {
            close_date: indicator.close_date,
            disclosed_date: indicator.disclosed_date,
            mix,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::indicator::Indicator;
    use crate::domain::entity::indicator_analysis::{IndicatorAnalysis, IndicatorAnalysisSet};

    #[test]
    fn indicator_analysis_test() {
        let indicator = Indicator {
            close_date: Default::default(),
            disclosed_date: Default::default(),
            per: 0.0,
            pbr: 0.0,
            mix: 0.0,
        };
        let indicator_analysis_set = IndicatorAnalysisSet { indicator };
        let actual = IndicatorAnalysis::from(indicator_analysis_set);
        let expected = IndicatorAnalysis {
            close_date: Default::default(),
            disclosed_date: Default::default(),
            mix: Some(0.0),
        };
        assert_eq!(actual, expected);
    }
}
