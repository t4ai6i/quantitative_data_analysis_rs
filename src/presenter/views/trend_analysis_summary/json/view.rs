use anyhow::bail;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::presenter::presenters::trend_analysis::response;
use crate::presenter::views::high_low_direction_signal_analysis::view::HighLowDirectionSignalAnalysis;
use crate::presenter::views::sma_cos_analysis::view::SmaCosAnalysis;
use crate::presenter::views::trend_reversal_analysis::view::TrendReversalAnalysis;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct JsonRow {
    pub code: String,
    pub symbol: String,
    pub sma_cos_analysis: SmaCosAnalysis,
    pub high_low_direction_signal_analysis: HighLowDirectionSignalAnalysis,
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
                high_low_direction_signals,
                trend_reversal_analysis,
            } => {
                let sma_cos_analysis =
                    SmaCosAnalysis::from((crossover_pattern_filter, rate_of_chance, latest_chance));
                let high_low_direction_signal_analysis =
                    HighLowDirectionSignalAnalysis::from(high_low_direction_signals.as_slice());
                let trend_reversal_analysis = TrendReversalAnalysis::from(trend_reversal_analysis);
                Ok(JsonRow {
                    code: company.code.to_string(),
                    symbol: company.symbol.to_string(),
                    sma_cos_analysis,
                    high_low_direction_signal_analysis,
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
    use crate::domain::models::buy_sell_signal::model::BuySellSignal;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
    use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
    use crate::presenter::views::high_low_direction_signal_analysis::view::HighLowDirectionSignalAnalysis;
    use crate::presenter::views::sma_cos_analysis::view::SmaCosAnalysis;
    use crate::presenter::views::trend_analysis_summary::json::view::JsonRow;
    use crate::presenter::views::trend_reversal_analysis::view::TrendReversalAnalysis;
    use chrono::NaiveDate;
    use indoc::indoc;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "code": "8473",
              "symbol": "8473.T",
              "sma_cos_analysis": {
                "crossover_pattern": "Neither",
                "latest_chance": "2017-02-16",
                "rate_of_chance": 32.7
              },
              "high_low_direction_signal_analysis": [
                {
                  "type": "Stay",
                  "date": "2017-02-16"
                }
              ],
              "trend_reversal_analysis": {
                "type": "Buy",
                "sma_cos_date": "2023-08-15",
                "sma_cps_date": "2023-07-04",
                "body_engulfing_date": "2023-06-01",
                "ms_es_date": "2023-07-26",
                "macd_cos_date": "2023-08-15"
              }
            }"#};
        let sma_cos_analysis = SmaCosAnalysis {
            crossover_pattern: CrossoverPattern::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            rate_of_chance: 32.7,
        };
        let high_low_direction_signal_analysis = HighLowDirectionSignalAnalysis::from(
            [BuySellSignal {
                r#type: BuySellSignalType::Stay,
                date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            }]
            .as_slice(),
        );
        let trend_reversal_analysis = TrendReversalAnalysis {
            r#type: BuySellSignalType::Buy,
            sma_cos_date: NaiveDate::from_ymd_opt(2023, 8, 15).unwrap(),
            sma_cps_date: NaiveDate::from_ymd_opt(2023, 7, 4).unwrap(),
            body_engulfing_date: NaiveDate::from_ymd_opt(2023, 6, 1).unwrap(),
            ms_es_date: NaiveDate::from_ymd_opt(2023, 7, 26).unwrap(),
            macd_cos_date: NaiveDate::from_ymd_opt(2023, 8, 15).unwrap(),
        };
        let json_row = JsonRow {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            sma_cos_analysis,
            high_low_direction_signal_analysis,
            trend_reversal_analysis,
        };
        let actual = serde_json::to_string_pretty(&json_row).unwrap();
        assert_eq!(actual, json_str);
        let actual: JsonRow = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, json_row);
    }
}
