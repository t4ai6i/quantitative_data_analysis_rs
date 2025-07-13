use anyhow::Result;
use async_trait::async_trait;

use crate::domain::models::stock::model;
use crate::domain::models::stock::model::Stocks;
use crate::domain::repositories::stock::queries;

#[async_trait]
pub trait Stock {
    async fn get_stock<'a>(&self, query: queries::get_stock::Query<'a>) -> Result<model::Stock>;

    async fn get_stocks<'a>(&self, query: queries::get_stocks::Query<'a>) -> Result<Stocks>;
}
