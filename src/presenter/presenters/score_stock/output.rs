use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ScoreStock {
    pub code: String,
    pub scored_at: DateTime<Utc>,
    pub status: ScoreStatus,
    pub error_type: Option<ScoreErrorType>,
    pub score: Option<Score>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreStatus {
    Ok,
    Failed,
    Skipped,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreErrorType {
    InvalidInput,
    CalculationError,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Score {
    pub total: f64,
    pub components: BTreeMap<String, ComponentDetail>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ComponentDetail {
    pub raw: f64,
    pub normalized: f64,
    pub points: f64,
}
