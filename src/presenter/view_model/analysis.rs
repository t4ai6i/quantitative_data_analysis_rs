use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::cross_analysis::CrossAnalysis;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct Analysis {
    pub cross_analysis: CrossAnalysis,
    pub buy_sell_signal_analysis: BuySellSignalAnalysis,
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross::CrossDirectionType;
    use crate::domain::entity::price_action::BuySellSignalType;
    use crate::presenter::view_model::analysis::Analysis;
    use crate::presenter::view_model::buy_sell_signal_analysis::{
        BuySellSignal, BuySellSignalAnalysis,
    };
    use crate::presenter::view_model::cross_analysis::CrossAnalysis;
    use chrono::NaiveDate;
    use indoc::indoc;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "cross_analysis": {
                "code": "8473",
                "symbol": "8473.T",
                "cross_direction": "neither",
                "latest_chance": "2017-02-16",
                "chance_rate": 32.7
              },
              "buy_sell_signal_analysis": {
                "code": "8473",
                "symbol": "8473.T",
                "buy_sell_signals": [
                  {
                    "buy_sell_signal": "wait_and_see",
                    "date": "2017-02-16"
                  }
                ]
              }
            }"#};
        let cross_analysis = CrossAnalysis {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            cross_direction: CrossDirectionType::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            chance_rate: 32.7,
        };
        let buy_sell_signal_analysis = BuySellSignalAnalysis {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            buy_sell_signals: vec![BuySellSignal {
                buy_sell_signal: BuySellSignalType::WaitAndSee,
                date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            }],
        };
        let analysis = Analysis {
            cross_analysis,
            buy_sell_signal_analysis,
        };
        let actual: Analysis = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, analysis);
        let actual = serde_json::to_string_pretty(&analysis).unwrap();
        assert_eq!(actual, json_str);
    }
}
