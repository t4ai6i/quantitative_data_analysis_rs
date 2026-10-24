use rayon::prelude::*;

use crate::domain::models::stock::model;

pub trait Stocks {
    fn formatted_dates(&self, date_format: &str) -> Vec<String>;
    /// Adjusted OHLC in chart order: open, close, low, high.
    fn ohlcs(&self) -> Vec<f64>;
    /// Adjusted volume, including fractional values.
    fn volumes(&self) -> Vec<f64>;
}

impl Stocks for model::Stocks {
    fn formatted_dates(&self, date_format: &str) -> Vec<String> {
        self.par_iter()
            .map(|stock| stock.date.format(date_format).to_string())
            .collect()
    }

    fn ohlcs(&self) -> Vec<f64> {
        self.par_iter()
            .flat_map(|stock| {
                vec![
                    stock.adj_open,
                    stock.adj_close,
                    stock.adj_low,
                    stock.adj_high,
                ]
            })
            .collect()
    }

    fn volumes(&self) -> Vec<f64> {
        self.par_iter().map(|stock| stock.adj_volume).collect()
    }
}
