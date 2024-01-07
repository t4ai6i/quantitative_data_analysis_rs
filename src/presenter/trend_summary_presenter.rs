pub mod chart;
pub mod json;

use crate::domain::entity::cross::CrossDirectionType;
use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;
use crate::utils::custom_date_format;
use anyhow::Result;
use chrono::NaiveDate;
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
pub struct JSON {
    pub(crate) code: String,
    pub(crate) symbol: String,
    pub(crate) cross_direction: CrossDirectionType,
    #[serde(with = "custom_date_format")]
    pub(crate) latest_chance: NaiveDate,
    pub(crate) chance_rate: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendSummaryResponse {
    Chart { body: String },
    JSON { data: Vec<JSON> },
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
    use crate::presenter::trend_summary_presenter::JSON;
    use chrono::NaiveDate;
    use indoc::indoc;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "code": "8473",
              "symbol": "8473.T",
              "cross_direction": "neither",
              "latest_chance": "2017-02-16",
              "chance_rate": 32.7
            }"#};
        let json = JSON {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            cross_direction: CrossDirectionType::Neither,
            latest_chance: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            chance_rate: 32.7,
        };
        let actual: JSON = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, json);
        let actual = serde_json::to_string_pretty(&json).unwrap();
        assert_eq!(actual, json_str);
    }
}
