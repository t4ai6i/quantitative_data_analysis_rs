use crate::presenter::presenters::fetch_scoring_data::output::FetchScoringData;

#[derive(Debug, Clone, PartialEq)]
pub struct ScoreStock {
    pub data: FetchScoringData,
    pub preset_name: String,
}

impl ScoreStock {
    pub fn new(data: FetchScoringData, preset_name: impl Into<String>) -> Self {
        Self {
            data,
            preset_name: preset_name.into(),
        }
    }
}
