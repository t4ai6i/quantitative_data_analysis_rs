use anyhow::{Context, Result};
use rayon::prelude::*;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};

use crate::domain::models::company::model;
use crate::domain::repositories::company::{queries, repository};
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;

#[async_trait::async_trait]
impl<T> repository::Company for Dsv<T>
where
    T: FromSlice<Item = model::RowCompany> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::RowCompany: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_row_company<'a>(
        &self,
        query: &queries::get_company::Query<'a>,
    ) -> Result<model::RowCompany> {
        let vec_row_company = self.get_vec_row_company().await?;
        vec_row_company
            .par_iter()
            .find_first(|company| company.code.iter().any(|code| code.eq(query.code)))
            .cloned()
            .with_context(|| {
                format!(
                    "Not found company: {:?}\n{}",
                    &query,
                    Backtrace::force_capture()
                )
            })
    }

    async fn get_vec_row_company(&self) -> Result<Vec<model::RowCompany>> {
        let processed = T::process_tabular_data(self.buffer.as_ref(), self.has_headers)?;
        Ok(processed)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::domain::models::company::model;
    use crate::domain::repositories::company::queries;
    use crate::domain::repositories::company::repository::Company;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::company::structures::internal::tsv;

    const TSV: &[u8] = include_bytes!("../../../../assets/companies.tsv");

    #[tokio::test]
    async fn get_row_company_test() -> anyhow::Result<()> {
        let repository = Dsv::<tsv::Structure>::new(false, Bytes::from(TSV));
        let query = queries::get_company::Query {
            code: "13080",
            ..Default::default()
        };
        let actual = repository.get_row_company(&query).await?;
        let expected = model::RowCompany {
            code: Some("13080".to_string()),
            name: Some("Listed Index Fund TOPIX".to_string()),
            market: Some("0109".to_string()),
            product_category: None,
            symbol: Some("13080.0109".to_string()),
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
