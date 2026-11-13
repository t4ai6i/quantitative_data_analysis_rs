use anyhow::{Context, Result, ensure};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::domain::models::statement::model;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use crate::shared::float::validate_value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Structure {
    pub code: String,
    pub disclosed_date: NaiveDate,
    #[serde(default)]
    pub current_fiscal_year_end_date: Option<NaiveDate>,
    pub bps: Option<f64>,
    pub eps: Option<f64>,
    pub annual_dividend_forecast: Option<f64>,
    pub net_sales: Option<f64>,
    pub opp: Option<f64>,
    pub orp: Option<f64>,
    pub profit: Option<f64>,
    pub equity: Option<f64>,
    pub total_assets: Option<f64>,
    #[serde(default)]
    pub disclosure_json: Option<String>,
}

impl From<model::Statement> for Structure {
    fn from(value: model::Statement) -> Self {
        let model::Statement {
            code,
            disclosed_date,
            current_fiscal_year_end_date,
            bps,
            eps,
            annual_dividend_forecast,
            net_sales,
            opp,
            orp,
            profit,
            equity,
            total_assets,
        } = value;

        Self {
            code,
            disclosed_date,
            current_fiscal_year_end_date,
            bps: validate_value(bps),
            eps: validate_value(eps),
            annual_dividend_forecast: validate_value(annual_dividend_forecast),
            net_sales: validate_value(net_sales),
            opp: validate_value(opp),
            orp: validate_value(orp),
            profit: validate_value(profit),
            equity: validate_value(equity),
            total_assets: validate_value(total_assets),
            disclosure_json: None,
        }
    }
}

impl TryFrom<model::RowStatement> for Structure {
    type Error = anyhow::Error;

    fn try_from(value: model::RowStatement) -> Result<Self> {
        Ok(Self {
            code: value.code,
            disclosed_date: value.disclosed_date.context("missing disclosed date")?,
            current_fiscal_year_end_date: value.current_fiscal_year_end_date,
            bps: value.bps,
            eps: value.eps,
            annual_dividend_forecast: value.annual_dividend_forecast,
            net_sales: value.net_sales,
            opp: value.opp,
            orp: value.orp,
            profit: value.profit,
            equity: value.equity,
            total_assets: value.total_assets,
            disclosure_json: value
                .disclosure
                .as_ref()
                .map(serde_json::to_string)
                .transpose()?,
        })
    }
}

impl TryFrom<Structure> for model::RowStatement {
    type Error = anyhow::Error;

    fn try_from(value: Structure) -> Result<Self> {
        let Structure {
            code,
            disclosed_date,
            current_fiscal_year_end_date,
            bps,
            eps,
            annual_dividend_forecast,
            net_sales,
            opp,
            orp,
            profit,
            equity,
            total_assets,
            disclosure_json,
        } = value;

        let disclosure: Option<model::FinancialDisclosure> = disclosure_json
            .as_deref()
            .filter(|value| !value.is_empty())
            .map(serde_json::from_str)
            .transpose()
            .context("invalid disclosure_json")?;
        if let Some(disclosure) = &disclosure {
            ensure!(disclosure.matches_code(&code), "disclosure code mismatch");
            ensure!(
                disclosure.disclosed_date == disclosed_date,
                "disclosure date mismatch"
            );
            ensure!(
                disclosure.current_fiscal_year_end == current_fiscal_year_end_date,
                "disclosure fiscal year end mismatch"
            );
            ensure!(
                disclosure.operating_profit == opp,
                "disclosure operating profit mismatch"
            );
        }
        Ok(Self {
            code,
            disclosed_date: Some(disclosed_date),
            current_fiscal_year_end_date,
            bps,
            eps,
            annual_dividend_forecast,
            net_sales,
            opp,
            orp,
            profit,
            equity,
            total_assets,
            disclosure,
        })
    }
}

impl FromSlice for Structure {
    type Deserialize = Structure;
    type Item = model::RowStatement;

    fn data_format() -> DataFormat {
        DataFormat::Tsv
    }
}
