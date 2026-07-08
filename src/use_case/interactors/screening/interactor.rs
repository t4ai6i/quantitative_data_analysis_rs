use crate::domain::models::screening::model::ScreeningResults;
use crate::domain::repositories;
use crate::use_case::interfaces::screening::input;
use crate::use_case::interfaces::screening::use_case;
use anyhow::{Result, bail};
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Screening<'a, E, SR, SMR, CR> {
    engine: &'a E,
    stock_repository: &'a SR,
    statement_repository: &'a SMR,
    company_repository: &'a CR,
}

impl<'a, E, SR, SMR, CR> Screening<'a, E, SR, SMR, CR> {
    pub fn new(
        engine: &'a E,
        stock_repository: &'a SR,
        statement_repository: &'a SMR,
        company_repository: &'a CR,
    ) -> Self {
        Self {
            engine,
            stock_repository,
            statement_repository,
            company_repository,
        }
    }

    pub fn stock_repository(&self) -> &'a SR {
        self.stock_repository
    }

    pub fn statement_repository(&self) -> &'a SMR {
        self.statement_repository
    }

    pub fn company_repository(&self) -> &'a CR {
        self.company_repository
    }
}

#[async_trait]
impl<E, SR, SMR, CR> use_case::ScreeningEngine for Screening<'_, E, SR, SMR, CR>
where
    E: use_case::ScreeningEngine + Send + Sync,
    SR: repositories::stock::repository::Stock + Send + Sync,
    CR: repositories::company::repository::Company + Send + Sync,
    SMR: repositories::statement::repository::Statement + Send + Sync,
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
