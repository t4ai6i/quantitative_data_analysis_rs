use crate::domain::entity::company::Company;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait CompanyRepository {
    async fn get_companies(&self) -> Result<Vec<Company>>;
    async fn get_company(&self, code: &str, market: &str) -> Result<Company>;
}
