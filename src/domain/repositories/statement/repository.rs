use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::statement::model;
use crate::domain::repositories::statement::queries;

#[async_trait]
pub trait Statement {
    async fn get_row_statement<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> Result<model::RowStatement>;

    async fn get_statement<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> Result<model::Statement> {
        let row_statement = self.get_row_statement(query).await?;
        TryFrom::try_from(row_statement)
    }
}
