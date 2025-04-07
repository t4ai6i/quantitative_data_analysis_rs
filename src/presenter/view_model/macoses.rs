use crate::domain::models::macos::model::{MACOSes, Pattern};
use charts_rs::NIL_VALUE;
use itertools::Itertools;

pub trait MACOSesExt {
    fn collect_vec_sma_25_close_macos(&self, pattern: Pattern) -> Vec<f32>;
    fn collect_vec_sma_25_volume_macos(&self, pattern: Pattern) -> Vec<f32>;
    fn get_value(target: Pattern, pattern: Pattern, value: Option<f64>) -> f32;
}

impl MACOSesExt for MACOSes {
    fn collect_vec_sma_25_close_macos(&self, pattern: Pattern) -> Vec<f32> {
        self.iter()
            .map(|macos| {
                Self::get_value(
                    macos.pattern_close_volume.close,
                    pattern,
                    macos.sma_set_25.map(|average| average.close),
                )
            })
            .collect_vec()
    }

    fn collect_vec_sma_25_volume_macos(&self, pattern: Pattern) -> Vec<f32> {
        self.iter()
            .map(|macos| {
                Self::get_value(
                    macos.pattern_close_volume.volume,
                    pattern,
                    macos.sma_set_25.map(|average| average.volume),
                )
            })
            .collect_vec()
    }

    fn get_value(lhs: Pattern, rhs: Pattern, value: Option<f64>) -> f32 {
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
