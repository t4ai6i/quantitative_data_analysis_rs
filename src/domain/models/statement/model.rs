use chrono::{NaiveDate, NaiveTime};
use deref_derive::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

/// Period interpretation of the API's actual operating-profit value.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum OperatingProfitPeriod {
    /// The API source does not establish a general cumulative or single-quarter meaning.
    Unspecified,
}

/// An unmodified J-Quants disclosure. Times are Japan Standard Time.
///
/// The API does not explicitly establish whether every operating profit value
/// is cumulative or for a single quarter. Period dates and source fields are
/// retained without deriving or classifying quarterly amounts.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct FinancialDisclosure {
    pub code: String,
    pub disclosed_date: NaiveDate,
    pub disclosed_time: NaiveTime,
    pub disclosure_number: String,
    pub document_type: String,
    pub current_period_type: Option<String>,
    pub current_period_start: Option<NaiveDate>,
    pub current_period_end: Option<NaiveDate>,
    pub current_fiscal_year_start: Option<NaiveDate>,
    pub current_fiscal_year_end: Option<NaiveDate>,
    pub next_fiscal_year_start: Option<NaiveDate>,
    pub next_fiscal_year_end: Option<NaiveDate>,
    pub operating_profit: Option<f64>,
    pub operating_profit_period: OperatingProfitPeriod,
    pub forecast_operating_profit: Option<f64>,
    pub next_forecast_operating_profit: Option<f64>,
    pub forecast_operating_profit_2q: Option<f64>,
    pub next_forecast_operating_profit_2q: Option<f64>,
    pub non_consolidated_operating_profit: Option<f64>,
    pub non_consolidated_forecast_operating_profit: Option<f64>,
    pub non_consolidated_next_forecast_operating_profit: Option<f64>,
    pub non_consolidated_forecast_operating_profit_2q: Option<f64>,
    pub non_consolidated_next_forecast_operating_profit_2q: Option<f64>,
}

impl FinancialDisclosure {
    /// J-Quants accepts a four-character code and returns its five-character form.
    pub fn matches_code(&self, code: &str) -> bool {
        self.code == code || (code.len() == 4 && self.code.strip_suffix('0') == Some(code))
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RowStatement {
    pub code: String,
    pub disclosed_date: Option<NaiveDate>,
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
    pub disclosure: Option<FinancialDisclosure>,
}

impl RowStatement {
    /// Retains the legacy FY projection for statement data without disclosure metadata.
    pub fn is_usable_full_year(&self) -> bool {
        self.disclosure.as_ref().is_none_or(|disclosure| {
            disclosure.current_period_type.as_deref() == Some("FY")
                && (self.net_sales.is_some() || self.eps.is_some())
        })
    }
}

/// Represents a financial statement with various financial metrics.
///
/// This struct is designed to encapsulate the key financial data points for
/// a company or entity, including net sales, profits, and assets. It can be
/// used to store financial data for reporting or analysis_result purposes.
///
/// # Fields
///
/// * `code` (`String`) - A unique identifier for the statement, such as a stock or company code.
/// * `disclosed_date` (`NaiveDate`) - The date when this financial statement was disclosed.
///   Represented with the type `NaiveDate` to handle dates without timezone information.
/// * `bps` (`f64`) - Book Value Per Share (BPS, 一株あたり純資産).
///   Represents the value of equity on a per-share basis.
/// * `eps` (`f64`) - Earnings Per Share (EPS, 一株あたり当期純利益).
///   Represents the company's profitability on a per-share basis.
/// * `annual_dividend_forecast` (`f64`) - Annual dividend forecast (年間配当予想).
///   Represents the expected dividend per share for the year.
/// * `net_sales` (`f64`) - Net sales (売上高).
///   Indicates the total revenue generated from sales, excluding returns, allowances, and discounts.
/// * `opp` (`f64`) - Operating profit (営業利益).
///   Indicates the profit a company makes from its core business operations.
/// * `orp` (`f64`) - Ordinary profit (経常利益).
///   Reflects the company's profit from usual business activities, including operating and non-operating incomes.
/// * `profit` (`f64`) - Net income (当期純利益).
///   The company's total profit after tax and other deductions.
/// * `equity` (`f64`) - Equity (純資産).
///   Represents the total value of the shareholders' stake in the company.
/// * `total_assets` (`f64`) - Total assets (総資産).
///   Represents the total value of everything the company owns.
///
/// # Derive Attributes
///
/// The following traits are derived for this struct:
/// * `Debug` - Enables debugging by allowing the struct to be formatted using the `{:?}` formatter.
/// * `Clone` - Allows the struct to be duplicated easily using `.clone()`.
/// * `PartialEq` - Enables equality comparison between instances of `Statement`.
/// * `PartialOrd` - Allows partial ordering of instances of `Statement`.
/// * `Default` - Provides a default implementation for creating an instance of `Statement` with default values.
/// ```
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Statement {
    pub code: String,
    pub disclosed_date: NaiveDate,
    pub current_fiscal_year_end_date: Option<NaiveDate>,
    pub bps: f64,
    pub eps: f64,
    pub annual_dividend_forecast: f64,
    pub net_sales: f64,
    pub opp: f64,
    pub orp: f64,
    pub profit: f64,
    pub equity: f64,
    pub total_assets: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct Statements(pub Vec<Statement>);

impl TryFrom<RowStatement> for Statement {
    type Error = anyhow::Error;

    fn try_from(value: RowStatement) -> Result<Self, Self::Error> {
        let RowStatement {
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
            disclosure: _,
        } = value;

        Ok(Self {
            disclosed_date: disclosed_date
                .ok_or_else(|| anyhow::anyhow!("code:{} disclosed_date is None", code.as_str()))?,
            current_fiscal_year_end_date,
            bps: bps.unwrap_or(f64::NAN),
            eps: eps.unwrap_or(f64::NAN),
            annual_dividend_forecast: annual_dividend_forecast.unwrap_or(f64::NAN),
            net_sales: net_sales.unwrap_or(f64::NAN),
            opp: opp.unwrap_or(f64::NAN),
            orp: orp.unwrap_or(f64::NAN),
            profit: profit.unwrap_or(f64::NAN),
            equity: equity.unwrap_or(f64::NAN),
            total_assets: total_assets.unwrap_or(f64::NAN),
            code,
        })
    }
}
