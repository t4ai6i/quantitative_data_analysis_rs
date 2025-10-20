use crate::domain::models::stock::model;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Structure {
    #[serde(rename = "Date")]
    pub date: NaiveDate,
    #[serde(rename = "Open")]
    pub open: f64,
    #[serde(rename = "High")]
    pub high: f64,
    #[serde(rename = "Low")]
    pub low: f64,
    #[serde(rename = "Close")]
    pub close: f64,
    #[serde(rename = "Adj Close")]
    pub adj_close: f64,
    #[serde(rename = "Volume")]
    pub volume: u64,
}

impl From<Structure> for model::RowStock {
    fn from(value: Structure) -> Self {
        let Structure {
            date,
            open,
            high,
            low,
            close,
            adj_close,
            volume,
        } = value;
        Self {
            date: Some(date),
            open: Some(open),
            high: Some(high),
            low: Some(low),
            close: Some(close),
            adj_close: Some(adj_close),
            volume: Some(volume),
        }
    }
}

impl FromSlice for Structure {
    type Deserialize = Structure;
    type Item = model::RowStock;

    fn data_format() -> DataFormat {
        DataFormat::Csv
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::stock::structures::internal::csv::Structure;

    const CSV_8473: &[u8] = include_bytes!("../../../../../../assets/8473.T.csv");

    #[test]
    fn vec_stock_test() {
        let vec_stock = Structure::from_slice::<true>(CSV_8473);
        assert_eq!(vec_stock.len(), 246);
    }
}
