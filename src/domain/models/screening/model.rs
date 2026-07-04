use crate::domain::models::company::model::Company;
use crate::domain::models::statement::model::Statement;
use crate::domain::models::stock::model::Stock;
use crate::shared::float::validate_value;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningCandidate {
    pub code: String,
    pub market: String,
    pub symbol: String,
    pub company_name: String,
}

impl From<&Company> for ScreeningCandidate {
    fn from(company: &Company) -> Self {
        ScreeningCandidate {
            code: company.code.clone(),
            market: company.market.clone(),
            symbol: company.symbol.clone(),
            company_name: company.name.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningMetrics {
    pub per: Option<f64>,
    pub pbr: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub roe: Option<f64>,
    pub sales_growth: Option<f64>,
}

impl From<(&Stock, &Statement)> for ScreeningMetrics {
    fn from((stock, statement): (&Stock, &Statement)) -> Self {
        let price = stock.adj_close;

        let per = validate_value(price / statement.eps);
        let pbr = validate_value(price / statement.bps);
        let roe = validate_value(statement.profit / statement.equity).map(|v| v * 100.0);

        ScreeningMetrics {
            per,
            pbr,
            dividend_yield: None,
            roe,
            sales_growth: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScoreBreakdown {
    pub per: Option<f64>,
    pub pbr: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub roe: Option<f64>,
    pub sales_growth: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningResult {
    pub candidate: ScreeningCandidate,
    pub metrics: ScreeningMetrics,
    pub score_breakdown: ScoreBreakdown,
    pub total_score: f64,
    pub rank: usize,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningResults(pub Vec<ScreeningResult>);
