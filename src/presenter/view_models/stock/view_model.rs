use rayon::prelude::*;

use crate::domain::models::stock::model;

pub trait Stocks {
    fn formatted_dates(&self, date_format: &str) -> Vec<String>;
    fn ohlces(&self) -> Vec<f64>;
    fn volumes(&self) -> Vec<u64>;
}

impl Stocks for model::Stocks {
    fn formatted_dates(&self, date_format: &str) -> Vec<String> {
        self.par_iter()
            .map(|stock| stock.date.format(date_format).to_string())
            .collect()
    }

    fn ohlces(&self) -> Vec<f64> {
        self.par_iter()
            .flat_map(|stock| vec![stock.open, stock.close, stock.low, stock.high])
            .collect()
    }

    fn volumes(&self) -> Vec<u64> {
        self.par_iter().map(|stock| stock.volume).collect()
    }
}
