use crate::domain::models::screening::scoring::ValueScorePolicy;
use crate::presenter::presenters::fetch_scoring_data::output::FetchScoringData;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreStock {
    pub data: FetchScoringData,
    pub policy: ValueScorePolicy,
    pub scored_at: DateTime<Utc>,
}

impl ScoreStock {
    pub fn new(data: FetchScoringData, policy: ValueScorePolicy, scored_at: DateTime<Utc>) -> Self {
        Self {
            data,
            policy,
            scored_at,
        }
    }
}
