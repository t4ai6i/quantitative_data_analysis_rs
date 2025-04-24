use crate::domain::models::indicator_analysis::model::IndicatorAnalysis;
use crate::presenter::view_models::buy_sell_signal_analysis::view_model::BuySellSignalAnalysis;
use crate::presenter::view_models::macos_analysis::MACOSAnalysis;
use crate::presenter::view_models::trend_reversal_analysis::TrendReversalAnalysis;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Analysis {
    pub code: String,
    pub symbol: String,
    pub macos_analysis: MACOSAnalysis,
    pub ecp1_analysis: BuySellSignalAnalysis,
    pub trend_reversal_analysis: TrendReversalAnalysis,
    pub indicator_analysis: IndicatorAnalysis,
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;

    use crate::domain::models::buy_sell_signal::model::BuySellSignal;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
    use crate::domain::models::indicator_analysis::model::IndicatorAnalysis;
    use crate::domain::models::macos::model::Pattern;
    use crate::presenter::view_models::analysis::view_model::Analysis;
    use crate::presenter::view_models::buy_sell_signal_analysis::view_model::BuySellSignalAnalysis;
    use crate::presenter::view_models::macos_analysis::MACOSAnalysis;
    use crate::presenter::view_models::trend_reversal_analysis::TrendReversalAnalysis;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "code": "8473",
              "symbol": "8473.T",
              "macos_analysis": {
                "pattern": "Neither",
                "latest_chance": "2017-02-16",
                "rate_of_chance": 32.7
              },
              "ecp1_analysis": [
                {
                  "type": "Stay",
                  "date": "2017-02-16"
                }
              ],
              "trend_reversal_analysis": {
                "type": "Buy",
                "macos_date": "2023-08-15",
                "ecp2_date": "2023-06-01",
                "msesp_date": "2023-07-26",
                "macps_date": "2023-07-04"
              },
              "indicator_analysis": {
                "close_date": "2023-08-15",
                "disclosed_date": "2023-06-01",
                "pbr": null,
                "per": null,
                "mix": null
              }
            }"#};
        let macos_analysis = MACOSAnalysis {
            pattern: Pattern::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            rate_of_chance: 32.7,
        };
        let ecp1_analysis = BuySellSignalAnalysis::from(
            [BuySellSignal {
                r#type: BuySellSignalType::Stay,
                date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            }]
            .as_slice(),
        );
        let trend_reversal_analysis = TrendReversalAnalysis {
            r#type: BuySellSignalType::Buy,
            macos_date: NaiveDate::from_ymd_opt(2023, 8, 15).unwrap(),
            ecp2_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            msesp_date: NaiveDate::from_ymd_opt(2023, 7, 26).unwrap(),
            macps_date: NaiveDate::from_ymd_opt(2023, 7, 4).unwrap(),
        };
        let indicator_analysis = IndicatorAnalysis {
            close_date: NaiveDate::from_ymd_opt(2023, 8, 15).unwrap(),
            disclosed_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            pbr: None,
            per: None,
            mix: None,
        };
        let analysis = Analysis {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            macos_analysis,
            ecp1_analysis,
            trend_reversal_analysis,
            indicator_analysis,
        };
        let actual = serde_json::to_string_pretty(&analysis).unwrap();
        assert_eq!(actual, json_str);
        let actual: Analysis = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, analysis);
    }
}
