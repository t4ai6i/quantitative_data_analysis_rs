use crate::domain::models::screening::model::ScreeningResults;
use crate::use_case::interfaces::screening::input;
use crate::use_case::interfaces::screening::use_case;
use anyhow::{bail, Result};
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Screening<'a, E> {
    engine: &'a E,
}

impl<'a, E> Screening<'a, E> {
    pub fn new(engine: &'a E) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl<E> use_case::ScreeningEngine for Screening<'_, E>
where
    E: use_case::ScreeningEngine + Send + Sync,
{
    async fn handle(&self, input: input::Screening) -> Result<ScreeningResults> {
        validate_input(&input)?;
        self.engine.handle(input).await
    }
}

fn validate_input(input: &input::Screening) -> Result<()> {
    if input.market.trim().is_empty() {
        bail!("market is empty");
    }
    if input.limit == 0 {
        bail!("limit must be greater than 0");
    }
    if input.preset_name.trim().is_empty() {
        bail!("preset name is empty");
    }
    Ok(())
}
