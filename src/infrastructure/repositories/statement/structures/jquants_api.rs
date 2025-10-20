use chrono::NaiveDate;
use serde_json::Value;

use crate::domain::models::statement::model;
use crate::shared::from_json_string_value::FromJsonStringValue;

pub struct Response<'a> {
    pub code: String,
    pub value: &'a Value,
}

impl From<Response<'_>> for model::RowStatement {
    fn from(value: Response<'_>) -> Self {
        let Response { code, value } = value;

        let disclosed_date = NaiveDate::from_json_string_value("DisclosedDate", value).ok();
        let eps = f64::from_json_string_value("EarningsPerShare", value).ok();
        let bps = f64::from_json_string_value("BookValuePerShare", value).ok();
        let net_sales = u64::from_json_string_value("NetSales", value).ok();
        let opp = u64::from_json_string_value("OperatingProfit", value).ok();
        let orp = u64::from_json_string_value("OrdinaryProfit", value).ok();
        let profit = u64::from_json_string_value("Profit", value).ok();
        let equity = u64::from_json_string_value("Equity", value).ok();
        let total_assets = u64::from_json_string_value("TotalAssets", value).ok();

        Self {
            code,
            disclosed_date,
            eps,
            bps,
            net_sales,
            opp,
            orp,
            profit,
            equity,
            total_assets,
        }
    }
}
