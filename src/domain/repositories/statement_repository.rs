use crate::domain::models::statement::model::Statement;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait StatementRepository {
    async fn get_statement(&self, code: &str) -> Result<Statement>;
}
