use crate::domain::entity::macos::{MACOSType, VecMACOS};
use charts_rs::NIL_VALUE;
use itertools::Itertools;

pub trait VecMACOSExt {
    fn collect_vec_sma_25_close_macos(&self, r#type: MACOSType) -> Vec<f32>;
    fn collect_vec_sma_25_volume_macos(&self, r#type: MACOSType) -> Vec<f32>;
    fn get_value(target: MACOSType, r#type: MACOSType, value: Option<f64>) -> f32;
}

impl VecMACOSExt for VecMACOS {
    fn collect_vec_sma_25_close_macos(&self, r#type: MACOSType) -> Vec<f32> {
        self.0
            .iter()
            .map(|macos| {
                Self::get_value(
                    macos.macos_set_5_25.close,
                    r#type,
                    macos.sma_set_25.map(|average| average.close),
                )
            })
            .collect_vec()
    }

    fn collect_vec_sma_25_volume_macos(&self, r#type: MACOSType) -> Vec<f32> {
        self.0
            .iter()
            .map(|macos| {
                Self::get_value(
                    macos.macos_set_5_25.volume,
                    r#type,
                    macos.sma_set_25.map(|average| average.volume),
                )
            })
            .collect_vec()
    }

    fn get_value(lhs: MACOSType, rhs: MACOSType, value: Option<f64>) -> f32 {
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
