use crate::domain::entity::buy_sell_signal::BuySellSignal;
use crate::domain::entity::macos::VecMACOS;
use crate::domain::entity::macps::MACPS;

pub struct TrendReversalAnalysisSet<'a> {
    pub ecp2s: &'a [BuySellSignal],
    pub msesps: &'a [BuySellSignal],
    pub macps: &'a MACPS,
    pub macoses: &'a VecMACOS,
}
