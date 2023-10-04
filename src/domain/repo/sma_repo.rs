use crate::domain::entity::sma::SMA;
use crate::domain::entity::stock::Stock;

pub trait SMARepo {
    fn collect_vec<const W: usize>(&self, stocks: &[Stock]) -> Vec<SMA>;
}
