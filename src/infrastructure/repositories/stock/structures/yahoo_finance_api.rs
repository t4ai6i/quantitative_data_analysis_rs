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
            // Yahoo-provided OHLC/volume are adopted without a second split adjustment.
            adj_open: Some(quote.open),
            adj_high: Some(quote.high),
            adj_low: Some(quote.low),
            adj_close: Some(quote.close),
            volume: Some(quote.volume),
            adj_volume: quote.volume.to_f64(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adopts_provider_values_without_dividend_or_second_split_adjustment() {
        let quote = Quote {
            timestamp: 1_598_616_000,
            open: 126.0125,
            high: 126.4425,
            low: 124.5775,
            close: 124.8075,
            adjclose: 120.9557,
            volume: 187_630_000,
        };
        let row = model::RowStock::from(Response(&quote));
        let stock = model::Stock::try_from(row).unwrap();
        assert_eq!(stock.adj_open, quote.open);
        assert_eq!(stock.adj_high, quote.high);
        assert_eq!(stock.adj_low, quote.low);
        assert_eq!(stock.adj_close, quote.close);
        assert_ne!(stock.adj_close, quote.adjclose);
        assert_eq!(stock.volume, quote.volume);
        assert_eq!(stock.adj_volume, 187_630_000.0);
    }
}
