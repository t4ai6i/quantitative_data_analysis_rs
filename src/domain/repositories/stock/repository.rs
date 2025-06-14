use crate::domain::models::stock::model;
use crate::domain::models::stock::model::Stocks;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait Stock {
    async fn get_stock(
        &self,
        code: &str,
        market: &str,
        target_date: NaiveDate,
    ) -> Result<model::Stock>;

    async fn get_stocks(
        &self,
        code: &str,
        market: &str,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Stocks>;
}
