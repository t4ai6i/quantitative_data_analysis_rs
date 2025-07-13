use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::company::model;
use crate::domain::repositories::company::queries;

#[async_trait]
pub trait Company {
    async fn get_company<'a>(
        &self,
        query: queries::get_company::Query<'a>,
    ) -> Result<model::Company>;
    async fn get_companies(&self) -> Result<Vec<model::Company>>;
}
