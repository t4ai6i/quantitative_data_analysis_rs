use crate::domain::models::company::model::Company;
use crate::domain::models::statement::model::Statement;
use crate::shared::float::validate_value;
use anyhow::bail;

const DOMESTIC_STOCK_PRODUCT_CATEGORY: &str = "011";
const TARGET_MARKETS: [&str; 3] = ["0111", "0112", "0113"];

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningCandidate {
    pub code: String,
    pub market: String,
    pub symbol: String,
    pub company_name: String,
}

impl TryFrom<&Company> for ScreeningCandidate {
    type Error = anyhow::Error;

    fn try_from(company: &Company) -> Result<Self, Self::Error> {
        let Some(product_category) = company.product_category.as_ref() else {
            bail!(
                "Missing product_category in company. code: {}",
                company.code
            );
        };
        if product_category != DOMESTIC_STOCK_PRODUCT_CATEGORY {
            bail!("Not domestic stock. code: {}", company.code);
        }
        if !is_target_market(company.market.as_str()) {
            bail!(
                "Not target market. code: {}, market: {}",
                company.code,
                company.market
            );
        }

        let Some(code) = normalize_code(company.code.as_str()) else {
            bail!("Invalid code. code: {}", company.code);
        };
        Ok(ScreeningCandidate {
            code,
            market: company.market.clone(),
            symbol: company.symbol.clone(),
            company_name: company.name.clone(),
        })
    }
}

fn is_target_market(market: &str) -> bool {
    TARGET_MARKETS
        .iter()
        .any(|target_market| target_market == &market)
}

pub(crate) fn normalize_code(code: &str) -> Option<String> {
    let code = code.trim().to_ascii_uppercase();

    match code.len() {
        4 if code.chars().all(|char| char.is_ascii_alphanumeric()) => Some(code),
        5 if code.ends_with('0') && code[..4].chars().all(|char| char.is_ascii_alphanumeric()) => {
            Some(code[..4].to_string())
        }
        _ => None,
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

impl From<(f64, &Statement)> for ScreeningMetrics {
    fn from((price, statement): (f64, &Statement)) -> Self {
        let per = validate_value(price / statement.eps);
        let pbr = validate_value(price / statement.bps);
        let dividend_yield =
            validate_value(statement.annual_dividend_forecast / price).map(|v| v * 100.0);
        let roe = validate_value(statement.profit / statement.equity).map(|v| v * 100.0);

        Self {
            per,
            pbr,
            dividend_yield,
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
