use serde::{Deserialize, Serialize};

use crate::domain::entity::candle_stick_pattern_cross_trend_analysis::CrossDateNearestSignalDate;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::cross_analysis::CrossAnalysis;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Analysis {
    pub code: String,
    pub symbol: String,
    pub cross_analysis: CrossAnalysis,
    pub ecp1_analysis: BuySellSignalAnalysis,
    pub ecp2_buy_golden: Option<CrossDateNearestSignalDate>,
    pub ecp2_sell_dead: Option<CrossDateNearestSignalDate>,
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;

    use crate::domain::entity::buy_sell_signal::BuySellSignalType;
    use crate::domain::entity::candle_stick_pattern_cross_trend_analysis::CrossDateNearestSignalDate;
    use crate::domain::entity::cross::CrossDirectionType;
    use crate::presenter::view_model::analysis::Analysis;
    use crate::presenter::view_model::buy_sell_signal::BuySellSignal;
    use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
    use crate::presenter::view_model::cross_analysis::CrossAnalysis;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "code": "8473",
              "symbol": "8473.T",
              "cross_analysis": {
                "cross_direction": "Neither",
                "latest_chance": "2017-02-16",
                "chance_rate": 32.7
              },
              "ecp1_analysis": [
                {
                  "type": "Stay",
                  "date": "2017-02-16"
                }
              ],
              "ecp2_buy_golden": {
                "cross_date": "2023-06-15",
                "signal_date": "2023-06-06"
              },
              "ecp2_sell_dead": null
            }"#};
        let cross_analysis = CrossAnalysis {
            cross_direction: CrossDirectionType::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            chance_rate: 32.7,
        };
        let ecp1_analysis = BuySellSignalAnalysis(vec![BuySellSignal {
            r#type: BuySellSignalType::Stay,
            date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
        }]);
        let ecp2_buy_golden = Some(CrossDateNearestSignalDate {
            signal_date: NaiveDate::from_ymd_opt(2023, 6, 6).unwrap(),
            cross_date: NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(),
        });
        let analysis = Analysis {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            cross_analysis,
            ecp1_analysis,
            ecp2_buy_golden,
            ecp2_sell_dead: None,
        };
        let actual = serde_json::to_string_pretty(&analysis).unwrap();
        assert_eq!(actual, json_str);
        let actual: Analysis = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, analysis);
    }
}
