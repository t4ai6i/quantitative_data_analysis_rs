use crate::domain::models::sma::model::SMAs;
use rayon::prelude::*;

pub trait SMAsExt {
    fn collect_average_close(&self) -> Vec<f32>;
    fn collect_average_volume(&self) -> Vec<f32>;
}

impl<const N: usize> SMAsExt for SMAs<N> {
    fn collect_average_close(&self) -> Vec<f32> {
        self.par_iter().map(|sma| sma.sma_n.close as _).collect()
    }

    fn collect_average_volume(&self) -> Vec<f32> {
        self.par_iter().map(|sma| sma.sma_n.volume as _).collect()
    }
}
