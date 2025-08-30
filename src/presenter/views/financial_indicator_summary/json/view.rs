use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::presenter::presenters::financial_indicator::response;
use crate::presenter::views::financial_indicator::view;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct JsonRow {
    pub code: String,
    pub symbol: String,
    pub financial_indicator: view::FinancialIndicator,
}

impl TryFrom<&response::FinancialIndicator> for JsonRow {
    type Error = anyhow::Error;

    fn try_from(value: &response::FinancialIndicator) -> Result<Self, Self::Error> {
        match value {
            response::FinancialIndicator::JSON {
                code,
                market,
                financial_indicator,
            } => Ok(Self {
                code: code.to_string(),
                symbol: market.to_string(),
                financial_indicator: view::FinancialIndicator::from(financial_indicator),
            }),
            // _ => bail!(
            //     "Invalid response::FinancialIndicator variant for JsonRow. {:?}",
            //     value
            // ),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct JsonRows(pub Vec<JsonRow>);

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct JSON(pub JsonRows);

impl From<response::FinancialIndicators> for JSON {
    fn from(value: response::FinancialIndicators) -> Self {
        let vec_json_row = value
            .par_iter()
            .filter_map(|financial_indicator| JsonRow::try_from(financial_indicator).ok())
            .collect();
        Self(JsonRows(vec_json_row))
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::domain::models::financial_indicator::model;
    use crate::presenter::views::financial_indicator::view;
    use crate::presenter::views::financial_indicator_summary::json::view::JsonRow;

    #[test]
    fn serde_test() {
        let json_str = indoc! {r#"
            {
              "code": "8473",
              "symbol": "8473.T",
              "financial_indicator": {
                "close_date": "2017-02-16",
                "disclosed_date": "2017-02-16",
                "pbr": 10.0,
                "per": 11.0,
                "oppr": null,
                "orpr": null,
                "pr": 12.0,
                "mix": 2.0
              }
            }"#};
        let financial_indicator = model::FinancialIndicator {
            close_date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            disclosed_date: NaiveDate::from_ymd_opt(2017, 2, 16).unwrap(),
            pbr: 10.0,
            per: 11.0,
            oppr: None,
            orpr: None,
            pr: Some(12.0),
            mix: Some(2.0),
        };
        let financial_indicator = view::FinancialIndicator::from(&financial_indicator);
        let json_row = JsonRow {
            code: "8473".to_string(),
            symbol: "8473.T".to_string(),
            financial_indicator,
        };
        let actual = serde_json::to_string_pretty(&json_row).unwrap();
        assert_eq!(actual, json_str);
        let actual: JsonRow = serde_json::from_str(json_str).unwrap();
        assert_eq!(actual, json_row);
    }
}
