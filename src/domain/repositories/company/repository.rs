use crate::domain::models::company::model;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Company {
    async fn get_companies(&self) -> Result<Vec<model::Company>>;
    async fn get_company(&self, code: &str, market: &str) -> Result<model::Company>;
}
