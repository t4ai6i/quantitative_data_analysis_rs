use yahoo_finance_api::YQuoteItem;

use crate::domain::models::company::model;

impl From<YQuoteItem> for model::RowCompany {
    fn from(value: YQuoteItem) -> Self {
        let code = value.symbol.split('.').next().map(String::from);
        Self {
            code,
            name: Some(value.long_name),
            market: Some(value.exchange),
            symbol: Some(value.symbol),
        }
    }
}
