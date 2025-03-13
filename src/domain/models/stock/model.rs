use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Stock {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: u64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct Stocks(Vec<Stock>);
