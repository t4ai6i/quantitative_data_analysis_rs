use crate::domain::models::stock::model;
use anyhow::{Context, Error};
use chrono::NaiveDate;
use num_traits::ToPrimitive;
use serde_json::Value;
use std::str::FromStr;

pub struct Response<'a>(pub &'a Value);

impl TryFrom<Response<'_>> for model::Stock {
    type Error = Error;

    fn try_from(value: Response<'_>) -> Result<Self, Self::Error> {
        let Response(value) = value;
        let date = value["Date"]
            .as_str()
            .with_context(|| "[Date] not found".to_string())
            .and_then(|date| {
                NaiveDate::from_str(date)
                    .with_context(|| format!("[Date] is invalid format: {}", date))
            })?;
        let open = value["Open"]
            .as_f64()
            .with_context(|| "[Open] not found".to_string())?;
        let high = value["High"]
            .as_f64()
            .with_context(|| "[High] not found".to_string())?;
        let low = value["Low"]
            .as_f64()
            .with_context(|| "[Low] not found".to_string())?;
        let close = value["Close"]
            .as_f64()
            .with_context(|| "[Close] not found".to_string())?;
        let adj_close = value["AdjustmentClose"]
            .as_f64()
            .with_context(|| "[AdjustmentClose] not found".to_string())?;
        let volume = value["Volume"]
            .as_f64()
            .with_context(|| "[Volume] not found".to_string())
            .and_then(|volume| {
                volume
                    .to_u64()
                    .with_context(|| format!("[Volume] is invalid format: {}", volume))
            })?;
        Ok(model::Stock {
            date,
            open,
            high,
            low,
            close,
            adj_close,
            volume,
        })
    }
}
