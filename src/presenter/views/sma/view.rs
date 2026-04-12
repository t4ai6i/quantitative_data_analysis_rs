use crate::domain::models::sma::model;
use rayon::prelude::*;

pub trait SMAs {
    fn sma_closes(&self) -> Vec<f32>;
    fn sma_volumes(&self) -> Vec<f32>;
}

impl<const N: usize> SMAs for model::SMAs<N> {
    fn sma_closes(&self) -> Vec<f32> {
        self.par_iter().map(|sma| sma.close.0 as _).collect()
    }

    fn sma_volumes(&self) -> Vec<f32> {
        self.par_iter().map(|sma| sma.volume.0 as _).collect()
    }
}
