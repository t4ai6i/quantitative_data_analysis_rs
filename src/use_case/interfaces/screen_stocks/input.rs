use crate::presenter::presenters::score_stock::output::ScoreStock;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenStocks {
    pub scores: Vec<ScoreStock>,
    pub preset_name: String,
    pub generated_at: DateTime<Utc>,
}

impl ScreenStocks {
    pub fn new(
        scores: Vec<ScoreStock>,
        preset_name: impl Into<String>,
        generated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            scores,
            preset_name: preset_name.into(),
            generated_at,
        }
    }
}
