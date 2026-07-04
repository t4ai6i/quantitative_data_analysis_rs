use anyhow::Result;

use crate::presenter::presenters::screening::{output, presenter, response};

pub struct Json;

impl presenter::Screening for Json {
    fn handle(&self, output: output::Screening) -> Result<response::Screening> {
        Ok(response::Screening::JSON {
            screening_results: output.screening_results,
        })
    }
}
