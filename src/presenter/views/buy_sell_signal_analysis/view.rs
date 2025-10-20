use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::domain::models::buy_sell_signal::model;
use crate::domain::models::buy_sell_signal::model::BuySellSignalType;
use crate::shared::custom_date_format;

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct BuySellSignal {
    pub r#type: BuySellSignalType,
    #[serde(with = "custom_date_format::primitive")]
    pub date: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct BuySellSignalAnalysis(Vec<BuySellSignal>);

impl From<&[model::BuySellSignal]> for BuySellSignalAnalysis {
    fn from(value: &[model::BuySellSignal]) -> Self {
        let vec_buy_sell_signal = value
            .par_iter()
            .map(|element| BuySellSignal {
                r#type: element.r#type,
                date: element.date,
            })
            .collect();
        Self(vec_buy_sell_signal)
    }
}
