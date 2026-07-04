use crate::presenter::presenters::screening::{output, response};
use anyhow::Result;

pub trait Screening {
    fn handle(&self, output: output::Screening) -> Result<response::Screening>;
}
