use anyhow::bail;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::presenter::presenters::trend_analysis::response;
use crate::presenter::views::buy_sell_signal_analysis::view::BuySellSignalAnalysis;
use crate::presenter::views::macos_analysis::view::MACOSAnalysis;
use crate::presenter::views::trend_reversal_analysis::view::TrendReversalAnalysis;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct JsonRow {
    pub code: String,
    pub symbol: String,
    pub macos_analysis: MACOSAnalysis,
    pub ecp1_analysis: BuySellSignalAnalysis,
    pub trend_reversal_analysis: TrendReversalAnalysis,
}

impl TryFrom<&response::TrendAnalysis> for JsonRow {
    type Error = anyhow::Error;

    fn try_from(value: &response::TrendAnalysis) -> Result<Self, Self::Error> {
        match value {
            response::TrendAnalysis::Json {
                company,
                crossover_pattern_filter,
                rate_of_chance,
                latest_chance,
                ecp1s,
                trend_reversal_analysis,
            } => {
                let macos_analysis =
                    MACOSAnalysis::from((crossover_pattern_filter, rate_of_chance, latest_chance));
                let ecp1_analysis = BuySellSignalAnalysis::from(ecp1s.as_slice());
                let trend_reversal_analysis = TrendReversalAnalysis::from(trend_reversal_analysis);
                Ok(JsonRow {
                    code: company.code.to_string(),
                    symbol: company.symbol.to_string(),
                    macos_analysis,
                    ecp1_analysis,
                    trend_reversal_analysis,
                })
            }
            _ => bail!(
                "Invalid response::TrendAnalysis variant for JsonRow. {:?}",
                value
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct JsonRows(pub Vec<JsonRow>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct JSON(pub JsonRows);

impl From<response::TrendAnalyses> for JSON {
    fn from(value: response::TrendAnalyses) -> Self {
        let vec_json_row = value
            .par_iter()
            .filter_map(|trend_analysis| JsonRow::try_from(trend_analysis).ok())
            .collect();
        Self(JsonRows(vec_json_row))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;

    use crate::domain::models::buy_sell_signal::model::BuySellSignal;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
    use crate::domain::models::macos::model::Pattern;
    use crate::presenter::views::buy_sell_signal_analysis::view::BuySellSignalAnalysis;
    use crate::presenter::views::macos_analysis::view::MACOSAnalysis;
    use crate::presenter::views::trend_analysis_summary::json::view::JsonRow;
    use crate::presenter::views::trend_reversal_analysis::view::TrendReversalAnalysis;

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
        let json_row = JsonRow {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            macos_analysis,
            ecp1_analysis,
            trend_reversal_analysis,
        };
        let actual = serde_json::to_string_pretty(&json_row).unwrap();
        assert_eq!(actual, json_str);
        let actual: JsonRow = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, json_row);
    }
}
