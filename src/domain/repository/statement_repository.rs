use crate::domain::entity::statement::Statement;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait StatementRepository {
    async fn get_statement(&self, code: &str) -> Result<Statement>;
}
