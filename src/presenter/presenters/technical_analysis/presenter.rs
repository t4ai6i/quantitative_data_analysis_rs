use crate::presenter::presenters::technical_analysis::{output, response};
use anyhow::Result;

pub struct Json;

pub trait TechnicalAnalysis {
    fn handle(&self, output: output::TechnicalAnalysis) -> Result<response::TechnicalAnalysis>;
}

impl TechnicalAnalysis for Json {
    fn handle(&self, output: output::TechnicalAnalysis) -> Result<response::TechnicalAnalysis> {
        Ok(output)
    }
}
