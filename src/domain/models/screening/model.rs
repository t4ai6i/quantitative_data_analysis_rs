#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningCandidate {
    pub code: String,
    pub market: String,
    pub symbol: String,
    pub company_name: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ScreeningMetrics {
    pub per: Option<f64>,
    pub pbr: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub roe: Option<f64>,
    pub sales_growth: Option<f64>,
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
