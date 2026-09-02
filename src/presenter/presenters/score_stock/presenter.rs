use crate::presenter::presenters::score_stock::{output, response};
use anyhow::Result;

pub struct Json;

pub trait ScoreStock {
    fn handle(&self, output: output::ScoreStock) -> Result<response::ScoreStock>;
}

impl ScoreStock for Json {
    fn handle(&self, output: output::ScoreStock) -> Result<response::ScoreStock> {
        Ok(output)
    }
}
