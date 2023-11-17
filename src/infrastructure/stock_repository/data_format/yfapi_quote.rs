use crate::domain::entity::stock::{Stock, VecStock};
use chrono::NaiveDateTime;
use itertools::Itertools;
use yahoo_finance_api::Quote;

pub struct VecQuote(pub Vec<Quote>);

impl From<VecQuote> for VecStock {
    fn from(value: VecQuote) -> Self {
        let VecQuote(vec_quote) = value;
        let vec_stock = vec_quote
            .iter()
            .map(|quote| {
                let date =
                    NaiveDateTime::from_timestamp_opt(quote.timestamp as u32 as i64, 0).unwrap();
                Stock {
                    date: date.date(),
                    open: quote.open,
                    high: quote.high,
                    low: quote.low,
                    close: quote.close,
                    adj_close: quote.adjclose,
                    volume: quote.volume,
                }
            })
            .collect_vec();
        Self(vec_stock)
    }
}
