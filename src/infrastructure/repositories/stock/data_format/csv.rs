use crate::domain::models::stock::model::Stock;
use crate::infrastructure::from_slice;
use crate::infrastructure::from_slice::FromSlice;
use chrono::NaiveDate;
use from_slice::DataFormat;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Csv {
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

impl From<Csv> for Stock {
    fn from(value: Csv) -> Self {
        let Csv {
            date,
            open,
            high,
            low,
            close,
            adj_close,
            volume,
        } = value;
        Self {
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

impl FromSlice for Csv {
    type Deserialize = Csv;
    type Item = Stock;

    fn data_format() -> DataFormat {
        DataFormat::CSV
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::from_slice::FromSlice;
    const CSV_8473: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");

    #[test]
    fn vec_stock_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        assert_eq!(vec_stock.len(), 246);
    }
}
