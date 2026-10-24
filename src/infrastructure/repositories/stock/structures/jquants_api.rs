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
        let adj_open = value["AdjO"].as_f64();
        let adj_high = value["AdjH"].as_f64();
        let adj_low = value["AdjL"].as_f64();
        let adj_close = value["AdjC"].as_f64();
        let volume = value["Vo"]
            .as_f64()
            .filter(|volume| volume.is_finite() && *volume >= 0.0 && volume.fract() == 0.0)
            .and_then(|volume| volume.to_u64());
        let adj_volume = value["AdjVo"].as_f64();
        model::RowStock {
            date,
            open,
            high,
            low,
            close,
            adj_open,
            adj_high,
            adj_low,
            adj_close,
            volume,
            adj_volume,
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn preserves_adjusted_ohlc_and_fractional_volume() {
        let data = json!({
            "Date": "2024-01-04", "O": 100.0, "H": 120.0, "L": 90.0,
            "C": 110.0, "Vo": 101.0,
            "AdjO": 50.0, "AdjH": 60.0, "AdjL": 45.0,
            "AdjC": 55.0, "AdjVo": 202.5
        });
        let row = model::RowStock::from(Response(&data));
        let stock = model::Stock::try_from(row).unwrap();
        assert_eq!((stock.open, stock.close, stock.volume), (100.0, 110.0, 101));
        assert_eq!(
            (
                stock.adj_open,
                stock.adj_high,
                stock.adj_low,
                stock.adj_close
            ),
            (50.0, 60.0, 45.0, 55.0)
        );
        assert_eq!(stock.adj_volume, 202.5);
    }

    #[test]
    fn missing_adjusted_values_are_not_replaced_with_raw_values() {
        let data = json!({
            "Date": "2024-01-04", "O": 100.0, "H": 120.0, "L": 90.0,
            "C": 110.0, "Vo": 101.0, "AdjC": 55.0
        });
        let row = model::RowStock::from(Response(&data));
        assert_eq!(row.adj_open, None);
        assert_eq!(row.adj_volume, None);
        assert!(model::Stock::try_from(row).is_err());
    }
}
