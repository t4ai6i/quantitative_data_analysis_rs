use crate::domain::models::scoring::model::ValueScorePolicy;
use crate::presenter::presenters::fetch_scoring_data::output::FetchScoringData;
use crate::presenter::presenters::score_stock::{presenter, response};
use crate::use_case::interfaces::score_stock::{input, use_case};
use anyhow::Result;
use chrono::{DateTime, Utc};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ScoreStock<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> ScoreStock<'a, I, P>
where
    I: use_case::ScoreStock,
    P: presenter::ScoreStock,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn score(
        &self,
        data: FetchScoringData,
        policy: ValueScorePolicy,
        scored_at: DateTime<Utc>,
    ) -> Result<response::ScoreStock> {
        let input = input::ScoreStock::new(data, policy, scored_at);
        let output = self.interactor.handle(input).await?;
        let output = self.presenter.handle(output)?;
        Ok(output)
    }
}
