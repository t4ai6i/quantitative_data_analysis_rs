use crate::presenter::presenters::fetch_scoring_data::output::FetchScoringData;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreStock {
    pub data: FetchScoringData,
    pub preset_name: String,
    pub scored_at: DateTime<Utc>,
}

impl ScoreStock {
    pub fn new(
        data: FetchScoringData,
        preset_name: impl Into<String>,
        scored_at: DateTime<Utc>,
    ) -> Self {
        Self {
            data,
            preset_name: preset_name.into(),
            scored_at,
        }
    }
}
