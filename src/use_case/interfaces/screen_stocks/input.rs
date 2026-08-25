use crate::presenter::presenters::score_stock::output::ScoreStock;

#[derive(Debug, Clone, PartialEq)]
pub struct ScreenStocks {
    pub scores: Vec<ScoreStock>,
    pub preset_name: String,
}

impl ScreenStocks {
    pub fn new(scores: Vec<ScoreStock>, preset_name: impl Into<String>) -> Self {
        Self {
            scores,
            preset_name: preset_name.into(),
        }
    }
}
