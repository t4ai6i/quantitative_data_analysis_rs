use crate::domain::models::buy_sell_signal::model::BuySellSignal as EntityBuySellSignal;
use crate::presenter::view_models::buy_sell_signal::BuySellSignal;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct BuySellSignalAnalysis(pub Vec<BuySellSignal>);

impl From<&[EntityBuySellSignal]> for BuySellSignalAnalysis {
    fn from(value: &[EntityBuySellSignal]) -> Self {
        let vec_buy_sell_signal = value
            .iter()
            .map(|element| BuySellSignal {
                r#type: element.r#type,
                date: element.date,
            })
            .collect_vec();
        Self(vec_buy_sell_signal)
    }
}
