use crate::domain::models::company::model::Company;
use crate::domain::models::screening::model::ScreeningCandidate;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock::queries::get_stocks_by_date;
use crate::domain::repositories::{company, stock};
use anyhow::Result;
use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct QueryScreener<'a, CR, SR> {
    company_repository: &'a CR,
    stock_repository: &'a SR,
}

impl<'a, CR, SR> QueryScreener<'a, CR, SR> {
    pub fn new(company_repository: &'a CR, stock_repository: &'a SR) -> Self {
        Self {
            company_repository,
            stock_repository,
        }
    }
}

impl<CR, SR> QueryScreener<'_, CR, SR>
where
    CR: company::repository::Company + Send + Sync,
    SR: stock::repository::Stock + Send + Sync,
{
    pub async fn fetch_universe_candidates(&self) -> Result<Vec<Result<ScreeningCandidate>>> {
        let companies = self.company_repository.get_companies().await?;
        Ok(build_universe_candidates(companies.as_slice()))
    }

    pub async fn fetch_base_date_stocks(&self, target_date: NaiveDate) -> Result<Stocks> {
        let query = get_stocks_by_date::Query { date: target_date };
        self.stock_repository.get_stocks_by_date(&query).await
    }
}

fn build_universe_candidates(companies: &[Company]) -> Vec<Result<ScreeningCandidate>> {
    companies
        .iter()
        .map(TryFrom::try_from)
        .collect::<Vec<Result<ScreeningCandidate>>>()
}

#[cfg(test)]
mod tests {
    use anyhow::bail;
    use async_trait::async_trait;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use std::sync::Mutex;

    use crate::domain::models::company::model::{Company, RowCompany};
    use crate::domain::models::stock::model::RowStock;
    use crate::domain::repositories::company;
    use crate::domain::repositories::company::queries::get_company::Query;
    use crate::domain::repositories::stock;
    use crate::use_case::interactors::screening::query_screener::build_universe_candidates;

    struct DummyCompanyRepository;

    #[async_trait]
    impl company::repository::Company for DummyCompanyRepository {
        async fn get_row_company<'a>(&self, query: &Query<'a>) -> anyhow::Result<RowCompany> {
            bail!("not used");
        }

        async fn get_vec_row_company(&self) -> anyhow::Result<Vec<RowCompany>> {
            bail!("not used");
        }
    }

    struct StubStockRepository {
        requested_date: Mutex<Option<NaiveDate>>,
        rows: Vec<RowStock>,
    }

    #[async_trait]
    impl stock::repository::Stock for StubStockRepository {
        async fn get_row_stock<'a>(
            &self,
            query: &stock::queries::get_stock::Query<'a>,
        ) -> anyhow::Result<RowStock> {
            bail!("not used");
        }

        async fn get_vec_row_stock<'a>(
            &self,
            query: &stock::queries::get_stocks::Query<'a>,
        ) -> anyhow::Result<Vec<RowStock>> {
            bail!("not used");
        }

        async fn get_vec_row_stock_by_date(
            &self,
            query: &stock::queries::get_stocks_by_date::Query,
        ) -> anyhow::Result<Vec<RowStock>> {
            self.requested_date.lock().unwrap().replace(query.date);
            Ok(self.rows.clone())
        }
    }

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
}
