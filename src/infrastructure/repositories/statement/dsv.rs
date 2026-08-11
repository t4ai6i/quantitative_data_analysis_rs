use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;
use anyhow::{Context, Result};
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};

#[async_trait::async_trait]
impl<T> repository::Statement for Dsv<T>
where
    T: FromSlice<Item = model::RowStatement> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::RowStatement: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_row_statement<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> Result<model::RowStatement> {
        let vec_row_statement = self.get_row_full_year_statements(query).await?;
        vec_row_statement.first().cloned().with_context(|| {
            format!(
                "Not found statement: {:?}\n{}",
                &query,
                Backtrace::force_capture(),
            )
        })
    }

    async fn get_row_full_year_statements<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> Result<Vec<model::RowStatement>> {
        let processed = T::process_tabular_data(self.buffer.as_ref(), self.has_headers)?;
        let mut filtered: Vec<model::RowStatement> = processed
            .into_iter()
            .filter(|statement| statement.code.eq(query.code))
            .collect();
        filtered.sort_by(|a, b| {
            let a_date = a.disclosed_date.unwrap_or_default();
            let b_date = b.disclosed_date.unwrap_or_default();
            b_date.cmp(&a_date)
        });
        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::statement::structures::internal::tsv;
    use bytes::Bytes;

    const SAMPLE_CODE: &str = "7203";
    const TSV: &[u8] = include_bytes!("../../../../assets/statements.tsv");

    #[tokio::test]
    async fn get_row_full_year_statements_returns_sorted_by_disclosed_date_desc()
    -> anyhow::Result<()> {
        let repository = Dsv::<tsv::Structure>::new(true, Bytes::from(TSV));
        let query = queries::get_statement::Query { code: SAMPLE_CODE };
        let actual = repository.get_row_full_year_statements(&query).await?;
        assert!(!actual.is_empty());
        for i in 0..actual.len() - 1 {
            let current_date = actual[i].disclosed_date.unwrap_or_default();
            let next_date = actual[i + 1].disclosed_date.unwrap_or_default();
            assert!(current_date >= next_date);
        }
        Ok(())
    }

    #[tokio::test]
    async fn get_row_statement_returns_latest() -> anyhow::Result<()> {
        let repository = Dsv::<tsv::Structure>::new(true, Bytes::from(TSV));
        let query = queries::get_statement::Query { code: SAMPLE_CODE };
        let actual = repository.get_row_statement(&query).await?;
        let all_statements = repository.get_row_full_year_statements(&query).await?;
        assert_eq!(actual.code, all_statements[0].code);
        assert_eq!(actual.disclosed_date, all_statements[0].disclosed_date);
        Ok(())
    }
}
