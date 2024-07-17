use crate::domain::entity::cross::Cross;
use crate::domain::entity::stock::Stock;

pub struct StocksCrossesPair<'a> {
    pub stocks: &'a [Stock],
    pub crosses: &'a [Cross],
}
