use anyhow::{Context, Result, bail, ensure};
use chrono::{NaiveDate, NaiveTime};
use serde_json::Value;
use std::str::FromStr;

use crate::domain::models::statement::model;
use crate::shared::float::validate_value;

pub struct Response<'a> {
    pub code: String,
    pub value: &'a Value,
}

fn optional_string<'a>(value: &'a Value, key: &str) -> Result<Option<&'a str>> {
    match value.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if text.is_empty() => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(_) => bail!("invalid string field {key}"),
    }
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str> {
    optional_string(value, key)?.with_context(|| format!("missing field {key}"))
}

fn optional_parsed<T: FromStr>(value: &Value, key: &str) -> Result<Option<T>>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    optional_string(value, key)?
        .map(|text| text.parse().with_context(|| format!("invalid field {key}")))
        .transpose()
}

fn optional_number(value: &Value, key: &str) -> Result<Option<f64>> {
    optional_parsed::<f64>(value, key)?
        .map(|number| validate_value(number).with_context(|| format!("non-finite field {key}")))
        .transpose()
}

impl TryFrom<Response<'_>> for model::RowStatement {
    type Error = anyhow::Error;

    fn try_from(value: Response<'_>) -> Result<Self> {
        let Response { code, value } = value;
        let disclosed_date = required_string(value, "DiscDate")?
            .parse::<NaiveDate>()
            .context("invalid field DiscDate")?;
        let disclosed_time = required_string(value, "DiscTime")?
            .parse::<NaiveTime>()
            .context("invalid field DiscTime")?;
        let current_fiscal_year_end_date = optional_parsed(value, "CurFYEn")?;
        let eps = optional_number(value, "EPS")?;
        let bps = optional_number(value, "BPS")?;
        // FY は DivAnn（実績配当）に入ることが多く、FDivAnn（予想配当）は空の場合がある。
        let dividend = optional_number(value, "DivAnn")?;
        let forecast_dividend = optional_number(value, "FDivAnn")?;
        let annual_dividend_forecast = dividend.or(forecast_dividend);
        let net_sales = optional_number(value, "Sales")?;
        let opp = optional_number(value, "OP")?;
        let orp = optional_number(value, "OdP")?;
        let profit = optional_number(value, "NP")?;
        let equity = optional_number(value, "Eq")?;
        let total_assets = optional_number(value, "TA")?;
        let disclosure = model::FinancialDisclosure {
            code: required_string(value, "Code")?.to_owned(),
            disclosed_date,
            disclosed_time,
            disclosure_number: required_string(value, "DiscNo")?.to_owned(),
            document_type: required_string(value, "DocType")?.to_owned(),
            current_period_type: optional_string(value, "CurPerType")?.map(str::to_owned),
            current_period_start: optional_parsed(value, "CurPerSt")?,
            current_period_end: optional_parsed(value, "CurPerEn")?,
            current_fiscal_year_start: optional_parsed(value, "CurFYSt")?,
            current_fiscal_year_end: current_fiscal_year_end_date,
            next_fiscal_year_start: optional_parsed(value, "NxtFYSt")?,
            next_fiscal_year_end: optional_parsed(value, "NxtFYEn")?,
            operating_profit: opp,
            operating_profit_period: model::OperatingProfitPeriod::Unspecified,
            forecast_operating_profit: optional_number(value, "FOP")?,
            next_forecast_operating_profit: optional_number(value, "NxFOP")?,
            forecast_operating_profit_2q: optional_number(value, "FOP2Q")?,
            next_forecast_operating_profit_2q: optional_number(value, "NxFOP2Q")?,
            non_consolidated_operating_profit: optional_number(value, "NCOP")?,
            non_consolidated_forecast_operating_profit: optional_number(value, "FNCOP")?,
            non_consolidated_next_forecast_operating_profit: optional_number(value, "NxFNCOP")?,
            non_consolidated_forecast_operating_profit_2q: optional_number(value, "FNCOP2Q")?,
            non_consolidated_next_forecast_operating_profit_2q: optional_number(
                value,
                "NxFNCOP2Q",
            )?,
        };

        ensure!(disclosure.matches_code(&code), "disclosure code mismatch");
        Ok(Self {
            code,
            disclosed_date: Some(disclosed_date),
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
            disclosure: Some(disclosure),
        })
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
            "DiscTime": "15:00:00", "Code": "84730", "DiscNo": "1", "DocType": "FYFinancialStatements_Consolidated_JP",
            "CurFYEn": "2026-03-31",
            "DivAnn": "170.0",
            "FDivAnn": ""
        });

        let actual = model::RowStatement::try_from(Response {
            code: "8473".to_string(),
            value: &value,
        })
        .unwrap();
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
            "DiscTime": "15:00:00", "Code": "84730", "DiscNo": "1", "DocType": "3QFinancialStatements_Consolidated_JP",
            "CurFYEn": "2024-03-31",
            "DivAnn": "",
            "FDivAnn": "160.0"
        });

        let actual = model::RowStatement::try_from(Response {
            code: "8473".to_string(),
            value: &value,
        })
        .unwrap();
        assert_eq!(
            actual.current_fiscal_year_end_date,
            NaiveDate::from_ymd_opt(2024, 3, 31)
        );
        assert_eq!(actual.annual_dividend_forecast, Some(160.0));
    }
}
