use crate::domain::models::stock::model;
use crate::shared::float::validate_value;
use crate::shared::from_json_string_value::FromJsonStringValue;
use chrono::NaiveDate;
use num_traits::ToPrimitive;
use serde_json::Value;

pub struct Response<'a>(pub &'a Value);

impl From<Response<'_>> for model::RowStock {
    fn from(value: Response<'_>) -> Self {
        let Response(value) = value;
        let date = NaiveDate::from_json_string_value("Date", value).ok();
        let open = value["O"].as_f64();
        let high = value["H"].as_f64();
        let low = value["L"].as_f64();
        let close = value["C"].as_f64();
        let adj_close = value["AdjC"].as_f64();
        let volume = value["Vo"].as_f64().and_then(|volume| volume.to_u64());
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

impl TryFrom<Response<'_>> for model::BaseDatePrice {
    type Error = anyhow::Error;

    fn try_from(value: Response<'_>) -> Result<Self, Self::Error> {
        let Response(value) = value;

        let code = String::from_json_string_value("Code", value)?;
        let adj_close = value["AdjC"].as_f64().and_then(validate_value);

        Ok(Self { code, adj_close })
    }
}
