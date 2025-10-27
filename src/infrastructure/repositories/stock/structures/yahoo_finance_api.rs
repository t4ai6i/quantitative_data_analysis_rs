use chrono::DateTime;
use num_traits::ToPrimitive;
use yahoo_finance_api::Quote;

use crate::domain::models::stock::model;

pub struct Response<'a>(pub &'a Quote);

impl From<Response<'_>> for model::RowStock {
    fn from(value: Response<'_>) -> Self {
        let Response(quote) = value;
        let date = quote.timestamp.to_i64().and_then(|timestamp| {
            DateTime::from_timestamp(timestamp, 0).map(|timestamp| timestamp.naive_utc().date())
        });
        model::RowStock {
            date,
            open: Some(quote.open),
            high: Some(quote.high),
            low: Some(quote.low),
            close: Some(quote.close),
            adj_close: Some(quote.adjclose),
            volume: Some(quote.volume),
        }
    }
}
