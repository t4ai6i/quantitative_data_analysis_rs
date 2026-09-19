use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;
use anyhow::Result;
use async_trait::async_trait;
use std::fmt::{Debug, Display};

#[async_trait]
impl<T> repository::Statement for Dsv<T>
where
    T: FromSlice<Item = model::RowStatement> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::RowStatement: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_row_statements<'a>(
        &self,
        query: &queries::get_statements::Query<'a>,
    ) -> Result<Vec<model::RowStatement>> {
        let processed = T::process_tabular_data(self.buffer.as_ref(), self.has_headers)?;

        let vec_row_statement: Vec<model::RowStatement> = match query.code {
            Some(code) => processed
                .into_iter()
                .filter(|statement| statement.code.eq(code))
                .collect(),
            // Noneの場合は、全ての行を返す
            None => processed,
        };

        Ok(vec_row_statement)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::statement::structures::internal::tsv;
    use bytes::Bytes;
    use chrono::NaiveDate;

    const SAMPLE_CODE: &str = "72030";
    const TSV: &[u8] = include_bytes!("../../../../assets/statements.tsv");

    #[tokio::test]
    async fn get_row_statement_returns_latest() -> anyhow::Result<()> {
        let repository = Dsv::<tsv::Structure>::new(true, Bytes::from(TSV));
        let query = queries::get_statements::Query {
            code: Some(SAMPLE_CODE),
        };
        let actual = repository.get_row_statements(&query).await?;
        assert_eq!(actual[0].code, SAMPLE_CODE);
        assert_eq!(
            actual[0].disclosed_date,
            NaiveDate::from_ymd_opt(2026, 5, 8)
        );
        let actual = actual.last().unwrap();
        assert_eq!(actual.code, SAMPLE_CODE);
        assert_eq!(actual.disclosed_date, NaiveDate::from_ymd_opt(2022, 5, 11));
        Ok(())
    }
}
