use crate::domain::models::company::model;
use anyhow::{Context, Error};
use yahoo_finance_api::YQuoteItem;

impl TryFrom<YQuoteItem> for model::Company {
    type Error = Error;

    fn try_from(value: YQuoteItem) -> Result<Self, Self::Error> {
        let code = value
            .symbol
            .split('.')
            .next()
            .with_context(|| format!("Cannot split quote.symbol: {}", value.symbol))?;
        Ok(model::Company {
            code: code.to_owned(),
            name: value.long_name,
            market: value.exchange,
            symbol: value.symbol,
        })
    }
}
