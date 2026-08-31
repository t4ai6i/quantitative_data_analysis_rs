use crate::presenter::presenters::fetch_scoring_data::{output, response};
use anyhow::Result;

pub struct Json;

pub trait FetchScoringData {
    fn handle(&self, output: output::FetchScoringData) -> Result<response::FetchScoringData>;
}

impl FetchScoringData for Json {
    fn handle(&self, output: output::FetchScoringData) -> Result<response::FetchScoringData> {
        Ok(output)
    }
}
