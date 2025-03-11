use chrono::NaiveDate;
use std::ops::{Deref, DerefMut};

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

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Stocks(Vec<Stock>);

impl Stocks {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl Deref for Stocks {
    type Target = Vec<Stock>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Stocks {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
