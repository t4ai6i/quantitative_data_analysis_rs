use crate::domain::models::statement::model::Statement;
use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use std::ops::Div;

/// 株価の指標を表す
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Indicator {
    /// 終値日時
    pub close_date: NaiveDate,
    /// 開示日時
    pub disclosed_date: NaiveDate,
    /// Price Book-Value Ratio
    pub pbr: f64,
    /// Price Earnings Ratio
    pub per: f64,
    /// Operating-Profit Ratio
    pub oppr: Option<f64>,
    /// Ordinary-Profit Ratio
    pub orpr: Option<f64>,
    /// Profit Ratio
    pub pr: Option<f64>,
}

impl From<(&Stock, &Statement)> for Indicator {
    fn from(value: (&Stock, &Statement)) -> Self {
        let (stock, statement) = value;
        let pbr = stock.close.div(statement.bps);
        let per = stock.close.div(statement.eps);
        let (oppr, orpr, pr) = (statement.net_sales != 0)
            .then(|| {
                let net_sales_f64 = statement.net_sales as f64;
                (
                    statement.opp as f64 / net_sales_f64 * 100.0,
                    statement.orp as f64 / net_sales_f64 * 100.0,
                    statement.profit as f64 / net_sales_f64 * 100.0,
                )
            })
            .map_or((None, None, None), |(oppr, orpr, pr)| {
                (Some(oppr), Some(orpr), Some(pr))
            });
        Self {
            close_date: stock.date,
            disclosed_date: statement.disclosed_date,
            per,
            pbr,
            oppr,
            orpr,
            pr,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::domain::models::indicator::model::Indicator;
    use crate::domain::models::statement::model::Statement;
    use crate::domain::models::stock::model::Stock;

    #[test]
    fn indicator_test() {
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 100.0,
            bps: 500.0,
            net_sales: 1000,
            opp: 500,
            orp: 200,
            profit: 100,
            ..Default::default()
        };
        let actual = Indicator::from((&stock, &statement));
        let expected = Indicator {
            per: 10.0,
            pbr: 2.0,
            oppr: Some(50.0),
            orpr: Some(20.0),
            pr: Some(10.0),
            ..Default::default()
        };
        assert_eq!(actual, expected);

        // ゼロ除算の確認
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let statement = Statement {
            bps: 0.0,
            eps: 100.0,
            net_sales: 0,
            ..Default::default()
        };
        let actual = Indicator::from((&stock, &statement));
        let expected = Indicator {
            per: 10.0,
            pbr: f64::INFINITY,
            oppr: None,
            orpr: None,
            pr: None,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
