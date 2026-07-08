use anyhow::Result;

use crate::domain::models::company::model::Company;
use crate::domain::models::screening::model::ScreeningCandidate;
use crate::domain::repositories::company::repository;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct QueryScreener<'a, CR> {
    company_repository: &'a CR,
}

impl<'a, CR> QueryScreener<'a, CR> {
    pub fn new(company_repository: &'a CR) -> Self {
        Self { company_repository }
    }
}

impl<CR> QueryScreener<'_, CR>
where
    CR: repository::Company + Send + Sync,
{
    pub async fn fetch_universe_candidates(&self) -> Result<Vec<Result<ScreeningCandidate>>> {
        let companies = self.company_repository.get_companies().await?;
        Ok(build_universe_candidates(companies.as_slice()))
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
    use pretty_assertions::assert_eq;

    use crate::domain::models::company::model::Company;
    use crate::use_case::interactors::screening::query_screener::build_universe_candidates;

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
