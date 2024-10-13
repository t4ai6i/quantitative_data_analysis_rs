use crate::domain::entity::macos::MACOS;
use crate::domain::entity::stock::Stock;

pub struct StocksMACOSESPair<'a> {
    pub stocks: &'a [Stock],
    pub macoses: &'a [MACOS],
}
