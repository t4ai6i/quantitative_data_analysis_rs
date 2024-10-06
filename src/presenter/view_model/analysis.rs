use serde::{Deserialize, Serialize};

use crate::domain::entity::candle_stick_pattern_date_set::CandleStickPatternDateSet;
use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
use crate::presenter::view_model::macos_analysis::MACOSAnalysis;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Analysis {
    pub code: String,
    pub symbol: String,
    pub macos_analysis: MACOSAnalysis,
    pub ecp1_analysis: BuySellSignalAnalysis,
    pub buy_candle_stick_pattern: Option<CandleStickPatternDateSet>,
    pub sell_candle_stick_pattern: Option<CandleStickPatternDateSet>,
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;

    use crate::domain::entity::buy_sell_signal::BuySellSignalType;
    use crate::domain::entity::candle_stick_pattern_date_set::CandleStickPatternDateSet;
    use crate::domain::entity::macos::MACOSType;
    use crate::presenter::view_model::analysis::Analysis;
    use crate::presenter::view_model::buy_sell_signal::BuySellSignal;
    use crate::presenter::view_model::buy_sell_signal_analysis::BuySellSignalAnalysis;
    use crate::presenter::view_model::macos_analysis::MACOSAnalysis;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "code": "8473",
              "symbol": "8473.T",
              "macos_analysis": {
                "type": "Neither",
                "latest_chance": "2017-02-16",
                "chance_rate": 32.7
              },
              "ecp1_analysis": [
                {
                  "type": "Stay",
                  "date": "2017-02-16"
                }
              ],
              "buy_candle_stick_pattern": {
                "macos_date": "2023-08-15",
                "ecp2_date": "2023-06-01",
                "msesp_date": "2023-07-26"
              },
              "sell_candle_stick_pattern": null
            }"#};
        let macos_analysis = MACOSAnalysis {
            r#type: MACOSType::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            chance_rate: 32.7,
        };
        let ecp1_analysis = BuySellSignalAnalysis(vec![BuySellSignal {
            r#type: BuySellSignalType::Stay,
            date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
        }]);
        let buy_candle_stick_pattern = Some(CandleStickPatternDateSet {
            macos_date: NaiveDate::from_ymd_opt(2023, 8, 15).unwrap(),
            ecp2_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            msesp_date: NaiveDate::from_ymd_opt(2023, 7, 26).unwrap(),
        });
        let analysis = Analysis {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            macos_analysis,
            ecp1_analysis,
            buy_candle_stick_pattern,
            sell_candle_stick_pattern: None,
        };
        let actual = serde_json::to_string_pretty(&analysis).unwrap();
        assert_eq!(actual, json_str);
        let actual: Analysis = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, analysis);
    }
}
