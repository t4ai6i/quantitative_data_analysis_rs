use crate::presenter::presenters::screening::{output, presenter, response};
use crate::use_case::interfaces::screening::{input, use_case};
use anyhow::Result;
use chrono::NaiveDate;

pub struct Screening<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> Screening<'a, I, P>
where
    I: use_case::ScreeningEngine,
    P: presenter::Screening,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        markets: Vec<String>,
        limit: usize,
        min_total_score: Option<usize>,
        preset_name: impl Into<String>,
        target_date: NaiveDate,
    ) -> Result<response::Screening> {
        let input =
            input::Screening::new(markets, limit, min_total_score, preset_name, target_date);
        let output = self.interactor.handle(input).await?;
        let output = output::Screening::new(output);
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
