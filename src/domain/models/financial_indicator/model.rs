use chrono::NaiveDate;

use crate::domain::models::statement::model::Statement;
use crate::domain::models::stock::model::Stock;

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
    pub oppr: f64,
    /// Ordinary-Profit Ratio/経常利益率
    pub orpr: f64,
    /// Profit Ratio/当期純利益率
    pub pr: f64,
    /// Mix Ratio/ミックス係数
    pub mix: f64,
    /// Return on Equity/自己資本利益率
    pub roe: f64,
    /// Return on Assets/総資産利益率
    pub roa: f64,
}

impl From<(&Stock, &Statement)> for FinancialIndicator {
    fn from(value: (&Stock, &Statement)) -> Self {
        let (stock, statement) = value;

        /*
           MIX係数 = PBR * PER
           https://zaimani.com/financial-indicators/mix-coefficient/
           純資産と当期純利益の両方で株価の割安性を測定する指標。
           提唱者ベンジャミン・グレアム氏曰く、ミックス係数が22.5を下回る銘柄が割安である。
           さらに手堅く見るならば、ミックス係数が2を下回る銘柄が割安である。
        */
        let pbr = if statement.bps.is_normal() {
            stock.close / statement.bps
        } else {
            f64::NAN
        };
        let per = if statement.eps.is_normal() {
            stock.close / statement.eps
        } else {
            f64::NAN
        };
        let mix = pbr * per;

        /*
           営業利益率における適正水準の目安
           【標準的な水準】10%以下
           【優良水準】11%～20%
           【高水準だが、注意が必要】20%以上
           https://www.kaonavi.jp/dictionary/eigyoriekiritsu/
        */
        let (oppr, orpr, pr) = if statement.net_sales.is_normal() {
            (
                statement.opp / statement.net_sales * 100.0,
                statement.orp / statement.net_sales * 100.0,
                statement.profit / statement.net_sales * 100.0,
            )
        } else {
            (f64::NAN, f64::NAN, f64::NAN)
        };

        /*
            ROE : 高ければ高いほど効率的に利益を稼いでいる（目安は8%）
            ROA : 高ければ高いほど効率的に利益を稼いでいる（目安は5%、ただし業種による変動幅がある）
            https://doda.jp/companyinfo/contents/finance/013.html
        */
        let roe = if statement.equity.is_normal() {
            statement.profit / statement.equity * 100.0
        } else {
            f64::NAN
        };
        let roa = if statement.total_assets.is_normal() {
            statement.profit / statement.total_assets * 100.0
        } else {
            f64::NAN
        };
        Self {
            close_date: stock.date,
            disclosed_date: statement.disclosed_date,
            per,
            pbr,
            oppr,
            orpr,
            pr,
            mix,
            roe,
            roa,
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
            code: "".to_string(),
            disclosed_date,
            eps: 100.0,
            bps: 400.0,
            annual_dividend_forecast: 0.0,
            net_sales: 512.0,
            opp: 128.0,
            orp: 64.0,
            profit: 32.0,
            equity: 1000.0,
            total_assets: 2000.0,
        };

        let actual = FinancialIndicator::from((&stock, &statement));
        let expected = FinancialIndicator {
            close_date,
            disclosed_date,
            per: 8.0,
            pbr: 2.0,
            mix: 16.0,
            oppr: 25.0,
            orpr: 12.5,
            pr: 6.25,
            roe: 3.2,
            roa: 1.6,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_net_sales_sets_profit_ratios_to_none_and_mix_still_computed_if_possible() {
        // 売上ゼロ -> 利益率は None
        // bps=0 -> pbr = NaN, per=10 -> mix = NaN
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 100.0, // per は有限
            bps: 0.0,   // pbr は ∞
            net_sales: 0.0,
            ..Default::default()
        };

        let actual = FinancialIndicator::from((&stock, &statement));
        assert_eq!(actual.per, 10.0);
        assert!(actual.pbr.is_nan());
        assert!(actual.mix.is_nan());
        assert!(actual.oppr.is_nan());
        assert!(actual.orpr.is_nan());
        assert!(actual.pr.is_nan());
    }

    #[test]
    fn mix_behaviour_with_nan_and_infinite_components() {
        // per が NaN, pbr 有限 -> mix = NaN
        let stock = Stock {
            close: 1000.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: f64::NAN, // per = NaN
            bps: 250.0,    // pbr = 4.0
            net_sales: 0.0,
            ..Default::default()
        };
        let actual = FinancialIndicator::from((&stock, &statement));
        assert!(actual.per.is_nan());
        assert_eq!(actual.pbr, 4.0);
        assert!(actual.mix.is_nan());
        assert!(actual.oppr.is_nan());
        assert!(actual.orpr.is_nan());
        assert!(actual.pr.is_nan());

        // pbr = NaN, per 有限 -> mix = NaN
        let stock = Stock {
            close: 1200.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 100.0, // per = 12
            bps: 0.0,   // pbr = NaN
            net_sales: 0.0,
            ..Default::default()
        };
        let actual = FinancialIndicator::from((&stock, &statement));
        assert_eq!(actual.per, 12.0);
        assert!(actual.pbr.is_nan());
        assert!(actual.mix.is_nan());
    }

    #[test]
    fn mix_is_infinite_when_both_per_and_pbr_are_infinite() {
        // eps=0 -> per = NaN, bps=0 -> pbr = NaN -> mix = NaN
        let stock = Stock {
            close: 500.0,
            ..Default::default()
        };
        let statement = Statement {
            eps: 0.0,
            bps: 0.0,
            net_sales: 0.0,
            ..Default::default()
        };

        let actual = FinancialIndicator::from((&stock, &statement));
        assert!(actual.per.is_nan());
        assert!(actual.pbr.is_nan());
        assert!(actual.mix.is_nan());
        assert!(actual.oppr.is_nan());
        assert!(actual.orpr.is_nan());
        assert!(actual.pr.is_nan());
    }

    #[test]
    fn roe_and_roa_none_when_denominators_zero() {
        // equity = 0 -> roe NaN, total_assets = 0 -> roa NaN
        let stock = Stock {
            close: 100.0,
            ..Default::default()
        };
        let statement = Statement {
            profit: 50.0,
            equity: 0.0,
            total_assets: 0.0,
            ..Default::default()
        };
        let actual = FinancialIndicator::from((&stock, &statement));
        assert!(actual.roe.is_nan());
        assert!(actual.roa.is_nan());
    }
}
