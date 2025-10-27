use chrono::NaiveDate;
use num_traits::ToPrimitive;
use serde_json::Value;

use crate::domain::models::stock::model;
use crate::shared::from_json_string_value::FromJsonStringValue;

pub struct Response<'a>(pub &'a Value);

impl From<Response<'_>> for model::RowStock {
    fn from(value: Response<'_>) -> Self {
        let Response(value) = value;
        let date = NaiveDate::from_json_string_value("Date", value).ok();
        let open = value["Open"].as_f64();
        let high = value["High"].as_f64();
        let low = value["Low"].as_f64();
        let close = value["Close"].as_f64();
        let adj_close = value["AdjustmentClose"].as_f64();
        let volume = value["Volume"].as_f64().and_then(|volume| volume.to_u64());
        model::RowStock {
            date,
            open,
            high,
            low,
            close,
            adj_close,
            volume,
        }
    }
}
