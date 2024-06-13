use crate::domain::entity::stock::{Stock, VecStock};
use chrono::NaiveDateTime;
use itertools::Itertools;
use yahoo_finance_api::Quote;

impl From<Vec<Quote>> for VecStock {
    fn from(value: Vec<Quote>) -> Self {
        let vec_stock = value
            .iter()
            .map(|quote| {
                let date =
                    NaiveDateTime::from_timestamp_opt(quote.timestamp as u32 as i64, 0).unwrap();
                Stock::new(
                    date.date(),
                    quote.open,
                    quote.high,
                    quote.low,
                    quote.close,
                    quote.adjclose,
                    quote.volume,
                )
            })
            .collect_vec();
        Self(vec_stock)
    }
}
