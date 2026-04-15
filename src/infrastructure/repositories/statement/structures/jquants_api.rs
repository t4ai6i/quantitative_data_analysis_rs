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

        let disclosed_date = NaiveDate::from_json_string_value("DiscDate", value).ok();
        let eps = f64::from_json_string_value("EPS", value).ok();
        let bps = f64::from_json_string_value("BPS", value).ok();
        let net_sales = f64::from_json_string_value("Sales", value).ok();
        let opp = f64::from_json_string_value("OP", value).ok();
        let orp = f64::from_json_string_value("OdP", value).ok();
        let profit = f64::from_json_string_value("NP", value).ok();
        let equity = f64::from_json_string_value("Eq", value).ok();
        let total_assets = f64::from_json_string_value("TA", value).ok();

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
