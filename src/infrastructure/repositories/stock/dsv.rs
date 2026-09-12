use anyhow::{Context, Result};
use rayon::prelude::*;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};

use crate::domain::models::stock::model;
use crate::domain::repositories::stock::queries::get_stocks_by_date::Query;
use crate::domain::repositories::stock::{queries, repository};
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;

#[async_trait::async_trait]
impl<T> repository::Stock for Dsv<T>
where
    T: FromSlice<Item = model::RowStock> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::RowStock: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_row_stock<'a>(
        &self,
        query: &queries::get_stock::Query<'a>,
    ) -> Result<model::RowStock> {
        let vec_row_stock = self
            .get_vec_row_stock(&queries::get_stocks::Query {
                ..Default::default()
            })
            .await?;
        vec_row_stock
            .par_iter()
            .find_first(|row_stock| {
                row_stock
                    .date
                    .iter()
                    .any(|date| date.eq(&query.target_date))
            })
            .cloned()
            .with_context(|| {
                format!(
                    "Not found stock: {:?}\n{}",
                    &query,
                    Backtrace::force_capture()
                )
            })
    }

    async fn get_vec_row_stock<'a>(
        &self,
        _: &queries::get_stocks::Query<'a>,
    ) -> Result<Vec<model::RowStock>> {
        let processed = T::process_tabular_data(self.buffer.as_ref(), self.has_headers)?;
        Ok(processed)
    }

    async fn get_vec_row_stock_by_date(&self, query: &Query) -> Result<Vec<model::RowStock>> {
        let vec_row_stock = self
            .get_vec_row_stock(&queries::get_stocks::Query::default())
            .await?;
        Ok(vec_row_stock
            .into_iter()
            .filter(|row_stock| row_stock.date.is_some_and(|date| date.eq(&query.date)))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use chrono::NaiveDate;

    use crate::domain::models::stock::model;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn get_row_stock_test() -> anyhow::Result<()> {
        let repository = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stock::Query {
            target_date: NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            ..Default::default()
        };
        let actual = repository.get_row_stock(&query).await?;
        let expected = model::RowStock {
            date: NaiveDate::from_ymd_opt(2023, 9, 8),
            open: Some(3100.0),
            high: Some(3129.0),
            low: Some(3100.0),
            close: Some(3119.0),
            adj_close: Some(3119.0),
            volume: Some(1571300),
        };
        assert_eq!(actual, expected,);
        Ok(())
    }
}
