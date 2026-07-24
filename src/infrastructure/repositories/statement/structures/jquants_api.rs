use chrono::NaiveDate;
use serde_json::Value;

use crate::domain::models::statement::model;
use crate::shared::float::validate_value;
use crate::shared::from_json_string_value::FromJsonStringValue;

pub struct Response<'a> {
    pub code: String,
    pub value: &'a Value,
}

impl From<Response<'_>> for model::RowStatement {
    fn from(value: Response<'_>) -> Self {
        let Response { code, value } = value;
        let parse_f64 = |key| {
            f64::from_json_string_value(key, value)
                .ok()
                .and_then(validate_value)
        };

        let disclosed_date = NaiveDate::from_json_string_value("DiscDate", value).ok();
        let eps = parse_f64("EPS");
        let bps = parse_f64("BPS");
        let annual_dividend_forecast = parse_f64("FDivAnn");
        let net_sales = parse_f64("Sales");
        let opp = parse_f64("OP");
        let orp = parse_f64("OdP");
        let profit = parse_f64("NP");
        let equity = parse_f64("Eq");
        let total_assets = parse_f64("TA");

        Self {
            code,
            disclosed_date,
            eps,
            bps,
            annual_dividend_forecast,
            net_sales,
            opp,
            orp,
            profit,
            equity,
            total_assets,
        }
    }
}
