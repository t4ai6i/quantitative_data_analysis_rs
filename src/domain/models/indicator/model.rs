use crate::domain::models::statement::model::Statement;
use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
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
}

impl From<(&Stock, &Statement)> for Indicator {
    fn from(value: (&Stock, &Statement)) -> Self {
        let (stock, statement) = value;
        let pbr = stock.close.div(statement.bps);
        let per = stock.close.div(statement.eps);
        Self {
            close_date: stock.date,
            disclosed_date: statement.disclosed_date,
            per,
            pbr,
        }
    }
}

#[cfg(test)]
mod tests {
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
            ..Default::default()
        };
        let actual = Indicator::from((&stock, &statement));
        let expected = Indicator {
            per: 10.0,
            pbr: 2.0,
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
            ..Default::default()
        };
        let actual = Indicator::from((&stock, &statement));
        let expected = Indicator {
            per: 10.0,
            pbr: f64::INFINITY,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
