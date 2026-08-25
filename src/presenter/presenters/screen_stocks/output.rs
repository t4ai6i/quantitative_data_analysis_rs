use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScreenStocks {
    pub generated_at: DateTime<Utc>,
    pub preset_name: String,
    pub summary: SummaryDetail,
    pub ranking: Vec<RankingEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SummaryDetail {
    pub total: usize,
    pub ok: usize,
    pub failed: usize,
    pub skipped: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RankingEntry {
    pub code: String,
    pub total_score: f64,
    pub components: BTreeMap<String, ComponentDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComponentDetail {
    pub raw: f64,
    pub normalized: f64,
    pub points: f64,
}
