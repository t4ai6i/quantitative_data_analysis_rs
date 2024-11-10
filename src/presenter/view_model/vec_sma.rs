use crate::domain::entity::sma::VecSMA;
use rayon::prelude::*;

pub trait VecSMAExt {
    fn collect_average_close(&self) -> Vec<f32>;
    fn collect_average_volume(&self) -> Vec<f32>;
}

impl<const N: usize> VecSMAExt for VecSMA<N> {
    fn collect_average_close(&self) -> Vec<f32> {
        self.0.par_iter().map(|sma| sma.sma_n.close as _).collect()
    }

    fn collect_average_volume(&self) -> Vec<f32> {
        self.0.par_iter().map(|sma| sma.sma_n.volume as _).collect()
    }
}
