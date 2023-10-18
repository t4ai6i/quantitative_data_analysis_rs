use chrono::NaiveDate;
use csv::ReaderBuilder;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Stock {
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
    pub volume: u32,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecStock<const B: bool>(pub Vec<Stock>);

impl<const B: bool> From<&[u8]> for VecStock<B> {
    ///
    /// # Examples
    /// ```ignore
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    /// let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
    /// assert_eq!(stocks.len(), 246);
    /// let VecStock::<true>(stocks) = CSV_8473.into();
    /// assert_eq!(stocks.len(), 246);
    /// ```
    fn from(value: &[u8]) -> Self {
        let mut reader = if B {
            ReaderBuilder::new().has_headers(true).from_reader(value)
        } else {
            ReaderBuilder::new().has_headers(false).from_reader(value)
        };
        let stocks = reader
            .deserialize::<Stock>()
            .filter_map(Result::ok)
            .collect_vec();
        Self(stocks)
    }
}

pub trait VecStockExt {
    fn collect_vec_day(&self) -> Vec<String>;
    fn collect_vec_candlestick(&self) -> Vec<f32>;
}

impl<const B: bool> VecStockExt for VecStock<B> {
    fn collect_vec_day(&self) -> Vec<String> {
        self.0
            .iter()
            .map(|stock| stock.date.format("%Y/%m/%d").to_string())
            .collect_vec()
    }

    fn collect_vec_candlestick(&self) -> Vec<f32> {
        self.0
            .iter()
            .flat_map(|stock| {
                vec![
                    stock.open as _,
                    stock.close as _,
                    stock.low as _,
                    stock.high as _,
                ]
            })
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn vec_stock_test() {
        let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
        assert_eq!(stocks.len(), 246);
        let VecStock::<true>(stocks) = CSV_8473.into();
        assert_eq!(stocks.len(), 246);
    }
}
