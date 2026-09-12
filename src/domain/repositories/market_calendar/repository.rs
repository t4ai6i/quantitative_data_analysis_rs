use crate::domain::repositories::market_calendar::queries;
use anyhow::Result;
use async_trait::async_trait;
use chrono::NaiveDate;

#[async_trait]
pub trait MarketCalendar {
    async fn get_previous_business_day(
        &self,
        query: &queries::get_previous_business_day::Query,
    ) -> Result<NaiveDate>;
}
