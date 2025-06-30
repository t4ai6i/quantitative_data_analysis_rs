use crate::domain::models::statement::model;
use anyhow::{bail, Context, Error};
use chrono::NaiveDate;
use serde_json::Value;
use std::str::FromStr;

pub struct Structure<'a> {
    pub code: String,
    pub value: &'a Value,
}

impl TryFrom<Structure<'_>> for model::Statement {
    type Error = Error;

    fn try_from(value: Structure<'_>) -> Result<Self, Self::Error> {
        let Structure { code, value } = value;
        let disclosed_date = value["DisclosedDate"]
            .as_str()
            .with_context(|| "[DisclosedDate] not found".to_string())
            .and_then(|disclosed_date| {
                NaiveDate::from_str(disclosed_date).with_context(|| {
                    format!("[DisclosedDate] is invalid format: {}", disclosed_date)
                })
            })?;
        // TypeOfCurrentPeriodはFY(Fiscal Year/事業年度)のみを対象とする
        let _ = value["TypeOfCurrentPeriod"]
            .as_str()
            .with_context(|| "[TypeOfCurrentPeriod] not found".to_string())
            .and_then(|type_of_current_period| {
                if type_of_current_period == "FY" {
                    Ok(type_of_current_period)
                } else {
                    bail!(
                        "[TypeOfCurrentPeriod] is not FY: {}",
                        type_of_current_period
                    )
                }
            })?;
        let eps = value["EarningsPerShare"]
            .as_str()
            .with_context(|| "[EarningsPerShare] not found".to_string())
            .and_then(|earnings_per_share| {
                earnings_per_share.parse::<f64>().with_context(|| {
                    format!(
                        "[EarningsPerShare] is invalid format: {}",
                        earnings_per_share
                    )
                })
            })?;
        let bps = value["BookValuePerShare"]
            .as_str()
            .with_context(|| "[BookValuePerShare] not found".to_string())
            .and_then(|book_value_per_share| {
                book_value_per_share.parse::<f64>().with_context(|| {
                    format!(
                        "[BookValuePerShare] is invalid format: {}",
                        book_value_per_share
                    )
                })
            })?;
        Ok(model::Statement {
            code,
            disclosed_date,
            eps,
            bps,
        })
    }
}
