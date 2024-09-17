use crate::domain::entity::buy_sell_signal::BuySellSignal;

pub struct CandleStickPatternSet<'a> {
    pub ecp2s: &'a [BuySellSignal],
    pub msesps: &'a [BuySellSignal],
}
