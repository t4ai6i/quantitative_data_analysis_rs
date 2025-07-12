use anyhow::{Context, Result};
use chrono::NaiveDate;
use rayon::prelude::*;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};

use crate::domain::models::stock::model;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock::repository;
use crate::infrastructure::dsv::Dsv;
use crate::infrastructure::from_slice::FromSlice;

#[async_trait::async_trait]
impl<T> repository::Stock for Dsv<T>
where
    T: FromSlice<Item = model::Stock> + Send + Sync,
    <T as FromSlice>::Deserialize: Send + Sync,
    <T::Item as TryFrom<T::Deserialize>>::Error: Debug + Display + Send + Sync,
    model::Stock: TryFrom<<T as FromSlice>::Deserialize>,
{
    async fn get_stock(&self, _: &str, _: &str, target_date: NaiveDate) -> Result<model::Stock> {
        let default_string = "".to_string();
        let stocks = self
            .get_stocks(
                default_string.as_str(),
                default_string.as_str(),
                NaiveDate::default(),
                NaiveDate::default(),
            )
            .await?;
        stocks
            .par_iter()
            .find_first(|stock| stock.date.eq(&target_date))
            .cloned()
            .with_context(|| {
                format!(
                    "Not found stock: {}\n{}",
                    target_date,
                    Backtrace::force_capture()
                )
            })
    }

    async fn get_stocks(&self, _: &str, _: &str, _: NaiveDate, _: NaiveDate) -> Result<Stocks> {
        let mut cache = self.cache.lock().await;
        if let Some(vec_stock) = cache.as_ref() {
            let mut vec_stock = vec_stock.clone();
            let mut stocks = Stocks::default();
            std::mem::swap(&mut vec_stock, &mut stocks);
            return Ok(stocks);
        }
        let buffer = self.buffer.as_slice();
        let mut processed = T::process_tabular_data(buffer, self.has_headers)?;
        *cache = Some(processed.clone());
        let mut stocks = Stocks::default();
        std::mem::swap(&mut processed, &mut stocks);
        Ok(stocks)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::domain::models::stock::model;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn get_stock_test() -> anyhow::Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, CSV.to_vec());
        let default_str = "";
        let actual = dsv
            .get_stock(
                default_str,
                default_str,
                NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
            )
            .await?;
        assert_eq!(
            actual,
            model::Stock {
                date: NaiveDate::from_ymd_opt(2023, 9, 8).unwrap(),
                open: 3100.0,
                high: 3129.0,
                low: 3100.0,
                close: 3119.0,
                adj_close: 3119.0,
                volume: 1571300,
            }
        );
        Ok(())
    }
}
