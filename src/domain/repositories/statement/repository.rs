use crate::domain::models::statement::model;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Statement {
    async fn get_statement(&self, code: &str) -> Result<model::Statement>;
}
