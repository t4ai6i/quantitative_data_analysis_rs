use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};

use crate::presenter::presenters::technical_analysis::{presenter, response};
use crate::use_case::interfaces::technical_analysis::{input, use_case};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TechnicalAnalysis<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TechnicalAnalysis<'a, I, P>
where
    I: use_case::TechnicalAnalysis,
    P: presenter::TechnicalAnalysis,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        analysis_at: DateTime<Utc>,
    ) -> Result<response::TechnicalAnalysis> {
        let input = input::TechnicalAnalysis::new(code, start_date, end_date, analysis_at);
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
