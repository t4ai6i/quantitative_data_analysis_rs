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
        let current_fiscal_year_end_date = NaiveDate::from_json_string_value("CurFYEn", value).ok();
        let eps = parse_f64("EPS");
        let bps = parse_f64("BPS");
        // FY は DivAnn（実績配当）に入ることが多く、FDivAnn（予想配当）は空の場合がある。
        let annual_dividend_forecast = parse_f64("DivAnn").or_else(|| parse_f64("FDivAnn"));
        let net_sales = parse_f64("Sales");
        let opp = parse_f64("OP");
        let orp = parse_f64("OdP");
        let profit = parse_f64("NP");
        let equity = parse_f64("Eq");
        let total_assets = parse_f64("TA");

        Self {
            code,
            disclosed_date,
            current_fiscal_year_end_date,
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

#[cfg(test)]
mod tests {
    use super::Response;
    use crate::domain::models::statement::model;
    use chrono::NaiveDate;
    use serde_json::json;

    #[test]
    fn annual_dividend_forecast_uses_divann_first() {
        let value = json!({
            "DiscDate": "2026-05-09",
            "CurFYEn": "2026-03-31",
            "DivAnn": "170.0",
            "FDivAnn": ""
        });

        let actual = model::RowStatement::from(Response {
            code: "8473".to_string(),
            value: &value,
        });
        assert_eq!(
            actual.current_fiscal_year_end_date,
            NaiveDate::from_ymd_opt(2026, 3, 31)
        );
        assert_eq!(actual.annual_dividend_forecast, Some(170.0));
    }

    #[test]
    fn annual_dividend_forecast_falls_back_to_fdivann() {
        let value = json!({
            "DiscDate": "2024-02-07",
            "CurFYEn": "2024-03-31",
            "DivAnn": "",
            "FDivAnn": "160.0"
        });

        let actual = model::RowStatement::from(Response {
            code: "8473".to_string(),
            value: &value,
        });
        assert_eq!(
            actual.current_fiscal_year_end_date,
            NaiveDate::from_ymd_opt(2024, 3, 31)
        );
        assert_eq!(actual.annual_dividend_forecast, Some(160.0));
    }
}
