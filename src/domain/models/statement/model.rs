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
    pub net_sales: usize,
    pub opp: usize,
    pub orp: usize,
    pub profit: usize,
    pub equity: usize,
    pub total_assets: usize,
}
