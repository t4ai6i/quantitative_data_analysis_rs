use crate::domain::models::stock::model::Stocks;
use itertools::Itertools;

pub trait StocksExt {
    fn collect_date_string(&self, date_format: &str) -> Vec<String>;
    fn collect_ohlc(&self) -> Vec<f64>;
    fn collect_volume(&self) -> Vec<u64>;
}

// TODO: rayonを使って並列化
impl StocksExt for Stocks {
    fn collect_date_string(&self, date_format: &str) -> Vec<String> {
        self.iter()
            .map(|stock| stock.date.format(date_format).to_string())
            .collect_vec()
    }

    fn collect_ohlc(&self) -> Vec<f64> {
        self.iter()
            .flat_map(|stock| vec![stock.open, stock.close, stock.low, stock.high])
            .collect_vec()
    }

    fn collect_volume(&self) -> Vec<u64> {
        self.iter().map(|stock| stock.volume).collect_vec()
    }
}
