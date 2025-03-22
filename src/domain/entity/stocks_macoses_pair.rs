use crate::domain::models::macos::model::MACOS;
use crate::domain::models::stock::model::Stock;

pub struct StocksMACOSESPair<'a> {
    pub stocks: &'a [Stock],
    pub macoses: &'a [MACOS],
}
