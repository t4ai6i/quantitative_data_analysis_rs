use chrono::NaiveDate;
use std::ops::Div;

use crate::domain::models::statement::model::Statement;
use crate::domain::models::stock::model::Stock;
use crate::shared::float::validate_value;

/// 株価や財務情報を元にした指標
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct FinancialIndicator {
    /// 終値日時
    pub close_date: NaiveDate,
    /// 開示日時
    pub disclosed_date: NaiveDate,
    /// Price Book-Value Ratio/株価純資産倍率
    pub pbr: f64,
    /// Price Earnings Ratio/株価収益率
    pub per: f64,
    /// Operating-Profit Ratio/営業利益率
    pub oppr: Option<f64>,
    /// Ordinary-Profit Ratio/経常利益率
    pub orpr: Option<f64>,
    /// Profit Ratio/当期純利益率
    pub pr: Option<f64>,
    /// Mix Ratio/ミックス係数
    pub mix: Option<f64>,
}

impl From<(&Stock, &Statement)> for FinancialIndicator {
    fn from(value: (&Stock, &Statement)) -> Self {
        let (stock, statement) = value;
        let pbr = stock.close.div(statement.bps);
        let per = stock.close.div(statement.eps);
        let per_option = Some(per).and_then(validate_value);
        let pbr_option = Some(pbr).and_then(validate_value);
        /*
           MIX係数 = PBR * PER
           https://zaimani.com/financial-indicators/mix-coefficient/
           純資産と当期純利益の両方で株価の割安性を測定する指標。
           提唱者ベンジャミン・グレアム氏曰く、ミックス係数が22.5を下回る銘柄が割安である。
           さらに手堅く見るならば、ミックス係数が2を下回る銘柄が割安である。
        */
        let mix = per_option.zip(pbr_option).map(|(pbr, per)| pbr * per);

        /*
           営業利益率における適正水準の目安
           【標準的な水準】10%以下
           【優良水準】11%～20%
           【高水準だが、注意が必要】20%以上
           https://www.kaonavi.jp/dictionary/eigyoriekiritsu/
        */
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
            mix,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::domain::models::financial_indicator::model::FinancialIndicator;
    use crate::domain::models::statement::model::Statement;
    use crate::domain::models::stock::model::Stock;

    #[test]
    fn computes_all_fields_and_propagates_dates() {
        // per = 800/100 = 8, pbr = 800/400 = 2, mix = 16
        // 利益率は 128/512*100 = 25.0, 64/512*100 = 12.5, 32/512*100 = 6.25
        let close_date = NaiveDate::from_ymd_opt(2024, 1, 2).unwrap();
        let disclosed_date = NaiveDate::from_ymd_opt(2023, 12, 31).unwrap();

        let stock = Stock {
            date: close_date,
            close: 800.0,
            ..Default::default()
        };
        let statement = Statement {
            disclosed_date,
            eps: 100.0,
            bps: 400.0,
            net_sales: 512,
            opp: 128,
            orp: 64,
            profit: 32,
            ..Default::default()
        };

        let actual = FinancialIndicator::from((&stock, &statement));
        let expected = FinancialIndicator {
            close_date,
            disclosed_date,
            per: 8.0,
            pbr: 2.0,
            mix: Some(16.0),
            oppr: Some(25.0),
            orpr: Some(12.5),
            pr: Some(6.25),
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_net_sales_sets_profit_ratios_to_none_and_pbr_infinite() {
        // 売上ゼロ -> 利益率は None
        // bps=0 -> pbr = ∞（mix は None になる想定）
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 100.0, // per は有限
            bps: 0.0,   // pbr は ∞
            net_sales: 0,
            ..Default::default()
        };

        let actual = FinancialIndicator::from((&stock, &statement));
        let expected = FinancialIndicator {
            per: 10.0,
            pbr: f64::INFINITY,
            mix: None,
            oppr: None,
            orpr: None,
            pr: None,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn mix_is_none_when_only_one_of_per_or_pbr_is_valid() {
        // per が NaN（無効）で pbr は有限 -> mix は None
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: f64::NAN, // per = NaN
            bps: 250.0,    // pbr = 4.0（有限）
            net_sales: 0,
            ..Default::default()
        };
        let actual = FinancialIndicator::from((&stock, &statement));

        // NaN は等値比較できないため、構造体ごとの assert_eq! は使わない
        assert!(actual.per.is_nan());
        assert_eq!(actual.pbr, 4.0);
        assert_eq!(actual.mix, None);
        assert_eq!(actual.oppr, None);
        assert_eq!(actual.orpr, None);
        assert_eq!(actual.pr, None);

        // pbr が ∞（無効）で per は有限 -> mix は None
        let stock = Stock {
            close: 1200.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 100.0, // per = 12.0（有限）
            bps: 0.0,   // pbr = ∞（無効）
            net_sales: 0,
            ..Default::default()
        };
        let actual = FinancialIndicator::from((&stock, &statement));

        assert_eq!(actual.per, 12.0);
        assert!(actual.pbr.is_infinite());
        assert_eq!(actual.mix, None);
        assert_eq!(actual.oppr, None);
        assert_eq!(actual.orpr, None);
        assert_eq!(actual.pr, None);
    }

    #[test]
    fn mix_is_none_when_both_per_and_pbr_are_invalid() {
        // eps=0 -> per = ∞, bps=0 -> pbr = ∞, いずれも無効 -> mix は None
        let stock = Stock {
            close: 500.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 0.0,
            bps: 0.0,
            net_sales: 0,
            ..Default::default()
        };

        let actual = FinancialIndicator::from((&stock, &statement));
        let expected = FinancialIndicator {
            per: f64::INFINITY,
            pbr: f64::INFINITY,
            mix: None,
            oppr: None,
            orpr: None,
            pr: None,
            ..Default::default()
        };
        assert_eq!(actual, expected);
    }
}
