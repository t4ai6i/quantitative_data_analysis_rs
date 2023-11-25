use crate::domain::entity::company::Company;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CompanyRepository {
    async fn get_company(&self, code: impl Into<String> + Send + Copy) -> Result<Company>;
}
