use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FetchScoringData {
    pub code: String,
    pub fetched_at: DateTime<Utc>,
    pub status: FetchStatus,
    pub error_type: Option<FetchErrorType>,
    pub company: Option<FetchCompany>,
    pub price: Option<FetchPrice>,
    pub latest_statement: Option<FetchLatestStatement>,
    pub full_year_sales: Vec<FetchFullYearSales>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchStatus {
    Ok,
    RateLimited,
    EmptyData,
    Failed,
    Skipped,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchErrorType {
    RateLimited,
    EmptyCompany,
    EmptyStock,
    EmptyStatement,
    InvalidCode,
    RepositoryError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
pub struct FetchCompany {
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct FetchPrice {
    pub date: Option<NaiveDate>,
    pub adj_close: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct FetchLatestStatement {
    pub disclosed_date: Option<NaiveDate>,
    pub eps: Option<f64>,
    pub bps: Option<f64>,
    pub annual_dividend_forecast: Option<f64>,
    pub profit: Option<f64>,
    pub equity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct FetchFullYearSales {
    pub current_fiscal_year_end_date: Option<NaiveDate>,
    pub disclosed_date: Option<NaiveDate>,
    pub net_sales: Option<f64>,
}
