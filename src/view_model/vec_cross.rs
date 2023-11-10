use crate::domain::entity::cross::{CrossDirectionType, VecCross};
use charts_rs::NIL_VALUE;
use itertools::Itertools;

pub trait VecCrossExt {
    fn collect_vec_sma_25_ave(&self, r#type: CrossDirectionType) -> Vec<f32>;
}

impl VecCrossExt for VecCross {
    fn collect_vec_sma_25_ave(&self, r#type: CrossDirectionType) -> Vec<f32> {
        self.0
            .iter()
            .map(|cross| {
                if cross.cross_direction_5_25.0.eq(&r#type) {
                    cross.sma_25_ave.unwrap() as _
                } else {
                    NIL_VALUE
                }
            })
            .collect_vec()
    }
}
