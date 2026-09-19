use crate::domain::models::statement::model;
use crate::domain::repositories::statement::queries;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Statement {
    async fn get_statements<'a>(
        &self,
        query: &queries::get_statements::Query<'a>,
    ) -> Result<model::Statements> {
        let row_statements = self.get_row_statements(query).await?;
        let vec_statement = row_statements
            .into_iter()
            .filter_map(|row_statement| TryFrom::try_from(row_statement).ok())
            .collect();
        Ok(model::Statements(vec_statement))
    }

    async fn get_row_statements<'a>(
        &self,
        query: &queries::get_statements::Query<'a>,
    ) -> Result<Vec<model::RowStatement>>;
}
