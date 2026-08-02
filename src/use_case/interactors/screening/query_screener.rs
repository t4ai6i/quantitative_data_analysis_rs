use crate::domain::models::company::model::Company;
use crate::domain::models::screening::model::{
    ScreeningCandidate, ScreeningMetrics, normalize_code,
};
use crate::domain::models::stock::model::BaseDatePrices;
use crate::domain::repositories::stock::queries::get_stocks_by_date;
use crate::domain::repositories::{company, statement, stock};
use anyhow::Result;
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct QueryScreener<'a, CR, SR, SMR> {
    company_repository: &'a CR,
    stock_repository: &'a SR,
    statement_repository: &'a SMR,
}

impl<'a, CR, SR, SMR> QueryScreener<'a, CR, SR, SMR> {
    pub fn new(
        company_repository: &'a CR,
        stock_repository: &'a SR,
        statement_repository: &'a SMR,
    ) -> Self {
        Self {
            company_repository,
            stock_repository,
            statement_repository,
        }
    }
}

impl<CR, SR, SMR> QueryScreener<'_, CR, SR, SMR>
where
    CR: company::repository::Company + Send + Sync,
    SR: stock::repository::Stock + Send + Sync,
    SMR: statement::repository::Statement + Send + Sync,
{
    pub async fn fetch_universe_candidates(&self) -> Result<Vec<Result<ScreeningCandidate>>> {
        let companies = self.company_repository.get_companies().await?;
        Ok(build_universe_candidates(companies.as_slice()))
    }

    pub async fn fetch_base_date_prices(
        &self,
        target_date: NaiveDate,
    ) -> Result<HashMap<String, Option<f64>>> {
        let query = get_stocks_by_date::Query { date: target_date };
        let prices = self.stock_repository.get_base_date_prices(&query).await?;
        build_price_map(&prices)
    }

    pub async fn fetch_financial_metrics(
        &self,
        code: &str,
        adj_close: f64,
    ) -> Result<ScreeningMetrics> {
        let query = statement::queries::get_statement::Query { code };
        let statement = self.statement_repository.get_statement(&query).await?;
        Ok(ScreeningMetrics::from((adj_close, &statement)))
    }
}

fn build_price_map(prices: &BaseDatePrices) -> Result<HashMap<String, Option<f64>>> {
    let mut price_map = HashMap::new();

    for price in prices.iter() {
        let Some(normalized) = normalize_code(price.code.as_str()) else {
            continue;
        };

        price_map
            .entry(normalized)
            .and_modify(|adj_close: &mut Option<f64>| {
                if adj_close.is_none() && price.adj_close.is_some() {
                    *adj_close = price.adj_close;
                }
            })
            .or_insert(price.adj_close);
    }

    Ok(price_map)
}

fn build_universe_candidates(companies: &[Company]) -> Vec<Result<ScreeningCandidate>> {
    companies
        .iter()
        .map(TryFrom::try_from)
        .collect::<Vec<Result<ScreeningCandidate>>>()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::domain::models::company::model::Company;
    use crate::domain::models::stock::model::{BaseDatePrice, BaseDatePrices};
    use crate::use_case::interactors::screening::query_screener::{
        build_price_map, build_universe_candidates,
    };

    #[test]
    fn build_universe_candidates_returns_errors_for_non_target_companies() {
        let companies = vec![
            Company {
                code: "13010".to_string(),
                name: "Prime Domestic".to_string(),
                market: "0111".to_string(),
                product_category: Some("011".to_string()),
                symbol: "1301".to_string(),
            },
            Company {
                code: "13020".to_string(),
                name: "Non Domestic Product".to_string(),
                market: "0111".to_string(),
                product_category: Some("012".to_string()),
                symbol: "1302".to_string(),
            },
            Company {
                code: "13030".to_string(),
                name: "Wrong Market".to_string(),
                market: "0109".to_string(),
                product_category: Some("011".to_string()),
                symbol: "1303".to_string(),
            },
        ];

        let actual = build_universe_candidates(&companies);
        assert_eq!(actual.len(), 3);

        assert!(actual[0].is_ok());
        assert_eq!(actual[0].as_ref().unwrap().code, "1301");
        assert_eq!(actual[0].as_ref().unwrap().market, "0111");
        assert_eq!(
            actual[1].as_ref().unwrap_err().to_string(),
            "Not domestic stock. code: 13020"
        );
        assert_eq!(
            actual[2].as_ref().unwrap_err().to_string(),
            "Not target market. code: 13030, market: 0109"
        );
    }

    #[test]
    fn build_universe_candidates_accepts_alphanumeric_code() {
        let companies = vec![
            Company {
                code: "137A0".to_string(),
                name: "Alphanumeric code".to_string(),
                market: "0112".to_string(),
                product_category: Some("011".to_string()),
                symbol: "137A".to_string(),
            },
            Company {
                code: "1304".to_string(),
                name: "Already normalized".to_string(),
                market: "0113".to_string(),
                product_category: Some("011".to_string()),
                symbol: "1304".to_string(),
            },
        ];

        let actual = build_universe_candidates(&companies);
        assert_eq!(actual.len(), 2);
        assert!(actual[0].is_ok());
        assert!(actual[1].is_ok());
        assert_eq!(actual[0].as_ref().unwrap().code, "137A");
        assert_eq!(actual[0].as_ref().unwrap().market, "0112");
        assert_eq!(actual[1].as_ref().unwrap().code, "1304");
        assert_eq!(actual[1].as_ref().unwrap().market, "0113");
    }

    #[test]
    fn build_universe_candidates_returns_error_for_invalid_code() {
        let companies = vec![
            Company {
                code: "13@A0".to_string(),
                name: "Invalid code".to_string(),
                market: "0112".to_string(),
                product_category: Some("011".to_string()),
                symbol: "13@A".to_string(),
            },
            Company {
                code: "1304".to_string(),
                name: "Already normalized".to_string(),
                market: "0113".to_string(),
                product_category: Some("011".to_string()),
                symbol: "1304".to_string(),
            },
        ];

        let actual = build_universe_candidates(&companies);
        assert_eq!(actual.len(), 2);
        assert_eq!(
            actual[0].as_ref().unwrap_err().to_string(),
            "Invalid code. code: 13@A0"
        );
        assert!(actual[1].is_ok());
        assert_eq!(actual[1].as_ref().unwrap().code, "1304");
        assert_eq!(actual[1].as_ref().unwrap().market, "0113");
    }

    #[test]
    fn build_price_map_normalizes_code_and_keeps_missing_price() {
        let prices = BaseDatePrices(vec![
            BaseDatePrice {
                code: "13010".to_string(),
                adj_close: Some(1000.0),
            },
            BaseDatePrice {
                code: "137A0".to_string(),
                adj_close: None,
            },
        ]);

        let map = build_price_map(&prices).unwrap();
        assert_eq!(map.get("1301"), Some(&Some(1000.0)));
        assert_eq!(map.get("137A"), Some(&None));
    }

    #[test]
    fn build_price_map_skips_invalid_code() {
        let prices = BaseDatePrices(vec![BaseDatePrice {
            code: "13@A0".to_string(),
            adj_close: Some(1000.0),
        }]);

        let map = build_price_map(&prices).unwrap();
        assert!(map.is_empty());
    }

    #[test]
    fn build_price_map_prefers_some_when_normalized_code_duplicated() {
        let prices = BaseDatePrices(vec![
            BaseDatePrice {
                code: "13010".to_string(),
                adj_close: None,
            },
            BaseDatePrice {
                code: "1301".to_string(),
                adj_close: Some(1000.0),
            },
        ]);

        let map = build_price_map(&prices).unwrap();
        assert_eq!(map.get("1301"), Some(&Some(1000.0)));
    }
}
