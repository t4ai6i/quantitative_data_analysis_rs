use crate::domain::entity::sma::VecSMA;
use itertools::Itertools;

pub trait VecSMAExt {
    fn collect_vec_ave(&self) -> Vec<f32>;
}

impl<const N: usize> VecSMAExt for VecSMA<N> {
    fn collect_vec_ave(&self) -> Vec<f32> {
        self.0.iter().map(|sma| sma.ave as f32).collect_vec()
    }
}
