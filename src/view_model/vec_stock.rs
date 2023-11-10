use crate::domain::entity::stock::VecStock;
use itertools::Itertools;

pub trait VecStockExt {
    fn collect_vec_day(&self) -> Vec<String>;
    fn collect_vec_candlestick(&self) -> Vec<f32>;
}

impl VecStockExt for VecStock {
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
