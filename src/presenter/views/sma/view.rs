use rayon::prelude::*;

use crate::domain::models::sma::model;

pub trait SMAs {
    fn sma_n_closes(&self) -> Vec<f32>;
    fn sma_n_volumes(&self) -> Vec<f32>;
}

impl<const N: usize> SMAs for model::SMAs<N> {
    fn sma_n_closes(&self) -> Vec<f32> {
        self.par_iter().map(|sma| sma.sma_n.close as _).collect()
    }

    fn sma_n_volumes(&self) -> Vec<f32> {
        self.par_iter().map(|sma| sma.sma_n.volume as _).collect()
    }
}
