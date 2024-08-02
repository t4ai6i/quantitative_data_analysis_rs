use crate::domain::entity::buy_sell_signal::BuySellSignal;
use crate::domain::entity::cross::Cross;

pub struct CandleStickPatternsCrossesSet<'a> {
    pub ecp2s: &'a [BuySellSignal],
    pub crosses: &'a [Cross],
}
