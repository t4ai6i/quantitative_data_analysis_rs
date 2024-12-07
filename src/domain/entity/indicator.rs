use crate::domain::entity::statement::Statement;
use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use std::ops::{Div, Mul};

/// 株価の指標を表す
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Indicator {
    /// 終値日時
    pub close_date: NaiveDate,
    /// 開示日時
    pub disclosed_date: NaiveDate,
    /// Price Earnings Ratio
    pub per: f64,
    /// Price Book-Value Ratio
    pub pbr: f64,
    /// MIX
    pub mix: f64,
}

impl From<(&[Stock], &Statement)> for Indicator {
    fn from(value: (&[Stock], &Statement)) -> Self {
        let (stocks, statement) = value;
        stocks
            .last()
            .map(|stock| {
                let per = stock.close.div(statement.eps);
                let pbr = stock.close.div(statement.bps);
                let mix = per.mul(pbr);
                Self {
                    close_date: stock.date,
                    disclosed_date: statement.disclosed_date,
                    per,
                    pbr,
                    mix,
                }
            })
            .unwrap_or(Indicator {
                mix: f64::NAN,
                ..Self::default()
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::indicator::Indicator;
    use crate::domain::entity::statement::Statement;
    use crate::domain::entity::stock::Stock;

    #[test]
    fn indicator_test() {
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let stocks = vec![stock];
        let statement = Statement {
            eps: 100.0,
            bps: 500.0,
            ..Default::default()
        };
        let actual = Indicator::from((stocks.as_slice(), &statement));
        let expected = Indicator {
            per: 10.0,
            pbr: 2.0,
            mix: 20.0,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
