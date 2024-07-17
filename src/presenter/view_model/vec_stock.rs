use crate::domain::entity::stock::VecStock;
use itertools::Itertools;

pub trait VecStockExt {
    fn collect_date_string(&self, date_format: &str) -> Vec<String>;
    fn collect_ohlc(&self) -> Vec<f64>;
    fn collect_volume(&self) -> Vec<u64>;
}

impl VecStockExt for VecStock {
    fn collect_date_string(&self, date_format: &str) -> Vec<String> {
        self.0
            .iter()
            .map(|stock| stock.date.format(date_format).to_string())
            .collect_vec()
    }

    fn collect_ohlc(&self) -> Vec<f64> {
        self.0
            .iter()
            .flat_map(|stock| {
                vec![
                    stock.open,
                    stock.close,
                    stock.low,
                    stock.high,
                ]
            })
            .collect_vec()
    }

    fn collect_volume(&self) -> Vec<u64> {
        self.0.iter().map(|stock| stock.volume).collect_vec()
    }
}
