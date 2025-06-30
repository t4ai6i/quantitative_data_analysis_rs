use crate::domain::models::stock::model;
use anyhow::{Context, Error};
use chrono::DateTime;
use num_traits::ToPrimitive;
use yahoo_finance_api::Quote;

pub struct Response<'a>(pub &'a Quote);

impl TryFrom<Response<'_>> for model::Stock {
    type Error = Error;

    fn try_from(value: Response<'_>) -> Result<Self, Self::Error> {
        let Response(quote) = value;
        let date = quote
            .timestamp
            .to_i64()
            .with_context(|| format!("Quote.timestamp cannot convert to i64. {}", quote.timestamp))
            .and_then(|timestamp| {
                DateTime::from_timestamp(timestamp, 0)
                    .with_context(|| format!("Quote.timestamp is invalid. {}", timestamp))
                    .map(|timestamp| timestamp.naive_utc().date())
            })?;
        Ok(model::Stock {
            date,
            open: quote.open,
            high: quote.high,
            low: quote.low,
            close: quote.close,
            adj_close: quote.adjclose,
            volume: quote.volume,
        })
    }
}
