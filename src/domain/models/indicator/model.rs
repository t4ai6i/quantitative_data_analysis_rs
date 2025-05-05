use crate::domain::models::statement::model::Statement;
use crate::domain::models::stock::model::Stock;
use chrono::NaiveDate;
use std::ops::{Div, Mul};

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
    /// MIX = PBR * PER
    /// https://www.nikkei.com/article/DGXZQOUB1184K0R11C22A0000000/
    pub mix: f64,
}

impl From<(&[Stock], &Statement)> for Indicator {
    fn from(value: (&[Stock], &Statement)) -> Self {
        let (stocks, statement) = value;
        stocks
            .last()
            .map(|stock| {
                let pbr = stock.close.div(statement.bps);
                let per = stock.close.div(statement.eps);
                let mix = pbr.mul(per);
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
    use crate::domain::models::indicator::model::Indicator;
    use crate::domain::models::statement::model::Statement;
    use crate::domain::models::stock::model::Stock;

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

        // ゼロ除算の確認
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let stocks = vec![stock];
        let statement = Statement {
            bps: 0.0,
            eps: 100.0,
            ..Default::default()
        };
        let actual = Indicator::from((stocks.as_slice(), &statement));
        let expected = Indicator {
            per: 10.0,
            pbr: f64::INFINITY,
            mix: f64::INFINITY,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
