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
        }
    }
}

impl From<Structure> for model::RowStatement {
    fn from(value: Structure) -> Self {
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
        } = value;

        Self {
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
        }
    }
}

impl FromSlice for Structure {
    type Deserialize = Structure;
    type Item = model::RowStatement;

    fn data_format() -> DataFormat {
        DataFormat::Tsv
    }
}
