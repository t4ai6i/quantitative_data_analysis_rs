use anyhow::Result;
use async_trait::async_trait;
use rayon::prelude::*;

use crate::domain::models::stock::model;
use crate::domain::repositories::stock::queries;

#[async_trait]
pub trait Stock {
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
            .filter_map(|row_stock| TryFrom::try_from(row_stock).ok())
            .collect::<Vec<model::Stock>>();
        Ok(model::Stocks(vec_stock))
    }

    async fn get_stocks_by_date(
        &self,
        query: &queries::get_stocks_by_date::Query,
    ) -> Result<model::Stocks> {
        let vec_row_stock = self.get_vec_row_stock_by_date(query).await?;
        let vec_stock = vec_row_stock
            .into_par_iter()
            .filter_map(|row_stock| TryFrom::try_from(row_stock).ok())
            .collect::<Vec<model::Stock>>();
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
