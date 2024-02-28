pub mod chart;
pub mod json;

use crate::domain::entity::cross::CrossDirectionType;
use crate::domain::entity::price_action::{BuySellSignal, BuySellSignalByPriceAction};
use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;
use crate::utils::custom_date_format;
use anyhow::Result;
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

pub struct TrendSummaryOutput {
    vec_trend_analysis_response: VecTrendAnalysisResponse,
}

impl TrendSummaryOutput {
    pub fn new(vec_trend_analysis_response: VecTrendAnalysisResponse) -> Self {
        Self {
            vec_trend_analysis_response,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct CrossAnalysisJSON {
    pub(crate) code: String,
    pub(crate) symbol: String,
    pub(crate) cross_direction: CrossDirectionType,
    #[serde(with = "custom_date_format")]
    pub(crate) latest_chance: NaiveDate,
    pub(crate) chance_rate: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignalJSON {
    pub(crate) buy_sell_signal: BuySellSignal,
    #[serde(with = "custom_date_format")]
    pub(crate) date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct BuySellSignalAnalysisJSON {
    pub(crate) code: String,
    pub(crate) symbol: String,
    pub(crate) buy_sell_signals: Vec<BuySellSignalJSON>,
}

impl From<&[BuySellSignalByPriceAction]> for BuySellSignalAnalysisJSON {
    fn from(value: &[BuySellSignalByPriceAction]) -> Self {
        let buy_sell_signals = value
            .iter()
            .map(|e| BuySellSignalJSON {
                buy_sell_signal: e.buy_sell_signal,
                date: e.date,
            })
            .collect_vec();
        Self {
            code: "".to_string(),
            symbol: "".to_string(),
            buy_sell_signals,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct AnalysisJSON {
    pub cross_analysis: CrossAnalysisJSON,
    pub buy_sell_signal_analysis: BuySellSignalAnalysisJSON,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendSummaryResponse {
    Chart { body: String },
    JSON { data: Vec<AnalysisJSON> },
}

pub trait TrendSummaryPresenter {
    fn handle(
        &self,
        output: TrendSummaryOutput,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendSummaryResponse>;
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross::CrossDirectionType;
    use crate::domain::entity::price_action::BuySellSignal;
    use crate::presenter::trend_summary_presenter::{
        AnalysisJSON, BuySellSignalAnalysisJSON, BuySellSignalJSON, CrossAnalysisJSON,
    };
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
        let cross_analysis = CrossAnalysisJSON {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            cross_direction: CrossDirectionType::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            chance_rate: 32.7,
        };
        let buy_sell_signal_analysis = BuySellSignalAnalysisJSON {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            buy_sell_signals: vec![BuySellSignalJSON {
                buy_sell_signal: BuySellSignal::WaitAndSee,
                date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            }],
        };
        let analysis_json = AnalysisJSON {
            cross_analysis,
            buy_sell_signal_analysis,
        };
        let actual: AnalysisJSON = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, analysis_json);
        let actual = serde_json::to_string_pretty(&analysis_json).unwrap();
        assert_eq!(actual, json_str);
    }
}
