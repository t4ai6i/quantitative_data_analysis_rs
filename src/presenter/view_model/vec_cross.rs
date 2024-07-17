use crate::domain::entity::cross::{CrossDirectionType, VecCross};
use charts_rs::NIL_VALUE;
use itertools::Itertools;

pub trait VecCrossExt {
    fn collect_vec_sma_25_close_average(&self, r#type: CrossDirectionType) -> Vec<f32>;
    fn collect_vec_sma_25_volume_average(&self, r#type: CrossDirectionType) -> Vec<f32>;
    fn get_value(target: CrossDirectionType, r#type: CrossDirectionType, value: Option<f64>)
        -> f32;
}

impl VecCrossExt for VecCross {
    fn collect_vec_sma_25_close_average(&self, r#type: CrossDirectionType) -> Vec<f32> {
        self.0
            .iter()
            .map(|cross| {
                Self::get_value(
                    cross.cross_direction_5_25.close_average,
                    r#type,
                    cross.sma_25_average.map(|average| average.close),
                )
            })
            .collect_vec()
    }

    fn collect_vec_sma_25_volume_average(&self, r#type: CrossDirectionType) -> Vec<f32> {
        self.0
            .iter()
            .map(|cross| {
                Self::get_value(
                    cross.cross_direction_5_25.volume_average,
                    r#type,
                    cross.sma_25_average.map(|average| average.volume),
                )
            })
            .collect_vec()
    }

    fn get_value(lhs: CrossDirectionType, rhs: CrossDirectionType, value: Option<f64>) -> f32 {
        if lhs.eq(&rhs) {
            if let Some(value) = value {
                value as _
            } else {
                NIL_VALUE
            }
        } else {
            NIL_VALUE
        }
    }
}
