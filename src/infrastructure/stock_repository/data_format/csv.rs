use crate::domain::entity::stock::{Stock, VecStock};
use chrono::NaiveDate;
use csv::ReaderBuilder;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct CSVFormat {
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

pub struct VecCSVFormat<const B: bool>(pub Vec<CSVFormat>);

impl<const B: bool> From<&[u8]> for VecCSVFormat<B> {
    ///
    /// # Examples
    /// ```ignore
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    /// let vec_csv_format = VecCSVFormat::<true>::from(CSV_8473);
    /// let VecStock(stocks) = VecStock::from(vec_csv_format);
    /// assert_eq!(stocks.len(), 246);
    /// let vec_csv_format: VecCSVFormat<true> = CSV_8473.into();
    /// let VecStock(stocks) = VecStock::from(vec_csv_format);
    /// assert_eq!(stocks.len(), 246);
    /// ```
    fn from(value: &[u8]) -> Self {
        let mut reader = if B {
            ReaderBuilder::new().has_headers(true).from_reader(value)
        } else {
            ReaderBuilder::new().has_headers(false).from_reader(value)
        };
        let stocks = reader
            .deserialize::<CSVFormat>()
            .filter_map(Result::ok)
            .collect_vec();
        Self(stocks)
    }
}

impl<const B: bool> From<VecCSVFormat<B>> for VecStock {
    fn from(value: VecCSVFormat<B>) -> Self {
        let VecCSVFormat(csv_formats) = value;
        let stocks = csv_formats
            .iter()
            .map(|csv_format| Stock {
                date: csv_format.date,
                open: csv_format.open,
                high: csv_format.high,
                low: csv_format.low,
                close: csv_format.close,
                adj_close: csv_format.adj_close,
                volume: csv_format.volume,
            })
            .collect_vec();
        VecStock(stocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CSV_8473: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[test]
    fn vec_stock_test() {
        let vec_csv_format = VecCSVFormat::<true>::from(CSV_8473);
        let VecStock(stocks) = VecStock::from(vec_csv_format);
        assert_eq!(stocks.len(), 246);
        let vec_csv_format: VecCSVFormat<true> = CSV_8473.into();
        let VecStock(stocks) = VecStock::from(vec_csv_format);
        assert_eq!(stocks.len(), 246);
    }
}
