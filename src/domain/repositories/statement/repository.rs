use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::statement::model;
use crate::domain::repositories::statement::queries;

#[async_trait]
pub trait Statement {
    async fn get_statement<'a>(
        &self,
        query: queries::get_statement::Query<'a>,
    ) -> Result<model::Statement>;
}
