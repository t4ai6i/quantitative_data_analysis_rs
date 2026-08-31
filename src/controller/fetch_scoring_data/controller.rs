use crate::presenter::presenters::fetch_scoring_data::{presenter, response};
use crate::use_case::interfaces::fetch_scoring_data::{input, use_case};
use anyhow::Result;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FetchScoringData<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> FetchScoringData<'a, I, P>
where
    I: use_case::FetchScoringData,
    P: presenter::FetchScoringData,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn fetch(
        &self,
        code: impl Into<String>,
        target_date: NaiveDate,
        fetched_at: DateTime<Utc>,
    ) -> Result<response::FetchScoringData> {
        let input = input::FetchScoringData::new(code.into(), target_date, fetched_at);
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
