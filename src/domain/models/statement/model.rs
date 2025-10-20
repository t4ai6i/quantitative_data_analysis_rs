use chrono::NaiveDate;

/// Represents a financial statement with various financial metrics.
///
/// This struct is designed to encapsulate the key financial data points for
/// a company or entity, including net sales, profits, and assets. It can be
/// used to store financial data for reporting or analysis purposes.
///
/// # Fields
///
/// * `code` (`String`) - A unique identifier for the statement, such as a stock or company code.
/// * `disclosed_date` (`NaiveDate`) - The date when this financial statement was disclosed.
///   Represented with the type `NaiveDate` to handle dates without timezone information.
/// * `eps` (`f64`) - Earnings Per Share (EPS, 一株あたり当期純利益).
///   Represents the company's profitability on a per-share basis.
/// * `bps` (`f64`) - Book Value Per Share (BPS, 一株あたり純資産).
///   Represents the value of equity on a per-share basis.
/// * `net_sales` (`usize`) - Net sales (売上高).
///   Indicates the total revenue generated from sales, excluding returns, allowances, and discounts.
/// * `opp` (`usize`) - Operating profit (営業利益).
///   Indicates the profit a company makes from its core business operations.
/// * `orp` (`usize`) - Ordinary profit (経常利益).
///   Reflects the company's profit from usual business activities, including operating and non-operating incomes.
/// * `profit` (`usize`) - Net income (当期純利益).
///   The company's total profit after tax and other deductions.
/// * `equity` (`usize`) - Equity (純資産).
///   Represents the total value of the shareholders' stake in the company.
/// * `total_assets` (`usize`) - Total assets (総資産).
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
    pub eps: f64,
    pub bps: f64,
    pub net_sales: u64,
    pub opp: u64,
    pub orp: u64,
    pub profit: u64,
    pub equity: u64,
    pub total_assets: u64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RowStatement {
    pub code: String,
    pub disclosed_date: Option<NaiveDate>,
    pub eps: Option<f64>,
    pub bps: Option<f64>,
    pub net_sales: Option<u64>,
    pub opp: Option<u64>,
    pub orp: Option<u64>,
    pub profit: Option<u64>,
    pub equity: Option<u64>,
    pub total_assets: Option<u64>,
}

impl TryFrom<RowStatement> for Statement {
    type Error = anyhow::Error;

    fn try_from(value: RowStatement) -> Result<Self, Self::Error> {
        let RowStatement {
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
        } = value;

        Ok(Self {
            code,
            disclosed_date: disclosed_date
                .ok_or_else(|| anyhow::anyhow!("disclosed_date is None"))?,
            eps: eps.ok_or_else(|| anyhow::anyhow!("eps is None"))?,
            bps: bps.ok_or_else(|| anyhow::anyhow!("bps is None"))?,
            net_sales: net_sales.ok_or_else(|| anyhow::anyhow!("net_sales is None"))?,
            // 現状、分析に必須ではないためNoneのときは0とする。
            opp: opp.unwrap_or(0),
            // 現状、分析に必須ではないためNoneのときは0とする。
            orp: orp.unwrap_or(0),
            profit: profit.ok_or_else(|| anyhow::anyhow!("profit is None"))?,
            equity: equity.ok_or_else(|| anyhow::anyhow!("equity is None"))?,
            total_assets: total_assets.ok_or_else(|| anyhow::anyhow!("total_assets is None"))?,
        })
    }
}
