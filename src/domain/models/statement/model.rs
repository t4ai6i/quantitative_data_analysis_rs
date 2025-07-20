use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Statement {
    pub code: String,
    /// DisclosedDate/開示日
    pub disclosed_date: NaiveDate,
    /// EarningsPerShare/一株あたり当期純利益
    pub eps: f64,
    /// BookValuePerShare/一株あたり純資産
    pub bps: f64,
    /// NetSales/売上高
    pub net_sales: usize,
    /// OperatingProfit/営業利益
    pub opp: usize,
    /// OrdinaryProfit/経常利益
    pub orp: usize,
    /// Profit/NetIncome/当期純利益
    pub profit: usize,
}
