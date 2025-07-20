use anyhow::Error;
use chrono::NaiveDate;
use serde_json::Value;

use crate::domain::models::statement::model;
use crate::shared::from_json_key::FromJsonKey;

pub struct Structure<'a> {
    pub code: String,
    pub value: &'a Value,
}

impl TryFrom<Structure<'_>> for model::Statement {
    type Error = Error;

    fn try_from(value: Structure<'_>) -> Result<Self, Self::Error> {
        let Structure { code, value } = value;

        let disclosed_date = NaiveDate::from_json_key("DisclosedDate", value)?;
        let eps = f64::from_json_key("EarningsPerShare", value).unwrap_or_default();
        let bps = f64::from_json_key("BookValuePerShare", value).unwrap_or_default();
        let net_sales = usize::from_json_key("NetSales", value).unwrap_or_default();
        let operating_profit = usize::from_json_key("OperatingProfit", value).unwrap_or_default();
        let ordinary_profit = usize::from_json_key("OrdinaryProfit", value).unwrap_or_default();
        let profit = usize::from_json_key("Profit", value).unwrap_or_default();

        Ok(model::Statement {
            code,
            disclosed_date,
            eps,
            bps,
            net_sales,
            opp: operating_profit,
            orp: ordinary_profit,
            profit,
        })
    }
}
