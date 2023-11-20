use crate::domain::entity::stock::Stock;
use crate::infrastructure::csv_ext::CsvExt;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct StockCsvRow {
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

impl From<StockCsvRow> for Stock {
    fn from(value: StockCsvRow) -> Self {
        let StockCsvRow {
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

impl CsvExt for StockCsvRow {
    type CSVFormat = StockCsvRow;
    type Item = Stock;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::csv_ext::CsvExt;
    const CSV_8473: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[test]
    fn vec_stock_test() {
        let vec_stock = StockCsvRow::from_slice::<true>(CSV_8473);
        assert_eq!(vec_stock.len(), 246);
    }
}
