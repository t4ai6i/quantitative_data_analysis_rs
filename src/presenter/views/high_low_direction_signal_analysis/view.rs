use crate::domain::models::buy_sell_signal::model;
use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::shared::custom_serde::naive_date;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct HighLowDirectionSignal {
    pub r#type: BuySellSignalType,
    #[serde(with = "naive_date::primitive")]
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct HighLowDirectionSignalAnalysis(Vec<HighLowDirectionSignal>);

impl From<&[model::BuySellSignal]> for HighLowDirectionSignalAnalysis {
    fn from(value: &[model::BuySellSignal]) -> Self {
        let vec_high_low_direction_signal = value
            .par_iter()
            .map(|element| HighLowDirectionSignal {
                r#type: element.r#type,
                date: element.date,
            })
            .collect();
        Self(vec_high_low_direction_signal)
    }
}
