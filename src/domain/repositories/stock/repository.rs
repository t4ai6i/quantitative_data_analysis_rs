use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use rayon::prelude::*;

use crate::domain::models::stock::model;
use crate::domain::repositories::stock::queries;

#[async_trait]
pub trait Stock {
    async fn get_base_date_prices(
        &self,
        query: &queries::get_stocks_by_date::Query,
    ) -> Result<model::BaseDatePrices> {
        bail!(
            "get_base_date_prices is not implemented. date: {}",
            query.date
        );
    }

    async fn get_stock<'a>(&self, query: &queries::get_stock::Query<'a>) -> Result<model::Stock> {
        let row_stock = self.get_row_stock(query).await?;
        TryFrom::try_from(row_stock)
    }

    async fn get_stocks<'a>(
        &self,
        query: &queries::get_stocks::Query<'a>,
    ) -> Result<model::Stocks> {
        let vec_row_stock = self.get_vec_row_stock(query).await?;
        let vec_stock = vec_row_stock
            .into_par_iter()
            .map(|row_stock| {
                model::Stock::try_from(row_stock)
                    .with_context(|| format!("Invalid stock row: {:?}", row_stock.date))
            })
            .collect::<Result<Vec<model::Stock>>>()?;
        Ok(model::Stocks(vec_stock))
    }

    async fn get_stocks_by_date(
        &self,
        query: &queries::get_stocks_by_date::Query,
    ) -> Result<model::Stocks> {
        let vec_row_stock = self.get_vec_row_stock_by_date(query).await?;
        let vec_stock = vec_row_stock
            .into_par_iter()
            .map(|row_stock| {
                model::Stock::try_from(row_stock)
                    .with_context(|| format!("Invalid stock row: {:?}", row_stock.date))
            })
            .collect::<Result<Vec<model::Stock>>>()?;
        Ok(model::Stocks(vec_stock))
    }

    async fn get_row_stock<'a>(
        &self,
        query: &queries::get_stock::Query<'a>,
    ) -> Result<model::RowStock>;

    async fn get_vec_row_stock<'a>(
        &self,
        query: &queries::get_stocks::Query<'a>,
    ) -> Result<Vec<model::RowStock>>;

    async fn get_vec_row_stock_by_date(
        &self,
        query: &queries::get_stocks_by_date::Query,
    ) -> Result<Vec<model::RowStock>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    struct Rows(Vec<model::RowStock>);

    #[async_trait]
    impl Stock for Rows {
        async fn get_row_stock<'a>(
            &self,
            _: &queries::get_stock::Query<'a>,
        ) -> Result<model::RowStock> {
            Ok(self.0[0])
        }

        async fn get_vec_row_stock<'a>(
            &self,
            _: &queries::get_stocks::Query<'a>,
        ) -> Result<Vec<model::RowStock>> {
            Ok(self.0.clone())
        }

        async fn get_vec_row_stock_by_date(
            &self,
            _: &queries::get_stocks_by_date::Query,
        ) -> Result<Vec<model::RowStock>> {
            Ok(self.0.clone())
        }
    }

    #[tokio::test]
    async fn conversion_errors_are_propagated_by_both_list_methods() {
        let valid = model::RowStock {
            date: NaiveDate::from_ymd_opt(2024, 1, 4),
            open: Some(10.0),
            high: Some(12.0),
            low: Some(9.0),
            close: Some(11.0),
            adj_open: Some(5.0),
            adj_high: Some(6.0),
            adj_low: Some(4.5),
            adj_close: Some(5.5),
            volume: Some(100),
            adj_volume: Some(200.5),
        };
        let query = queries::get_stocks_by_date::Query {
            date: valid.date.unwrap(),
        };
        let rows = Rows(vec![valid]);
        let stocks = rows
            .get_stocks(&queries::get_stocks::Query::default())
            .await
            .unwrap();
        assert_eq!(stocks[0].adj_volume, 200.5);
        for invalid in [
            model::RowStock {
                adj_open: None,
                ..valid
            },
            model::RowStock {
                adj_volume: None,
                ..valid
            },
            model::RowStock {
                adj_close: Some(f64::NAN),
                ..valid
            },
            model::RowStock {
                adj_volume: Some(-1.0),
                ..valid
            },
        ] {
            let rows = Rows(vec![valid, invalid]);
            assert!(
                rows.get_stocks(&queries::get_stocks::Query::default())
                    .await
                    .is_err()
            );
            assert!(rows.get_stocks_by_date(&query).await.is_err());
        }
    }
}
