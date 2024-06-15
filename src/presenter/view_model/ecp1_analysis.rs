use crate::domain::entity::ecp1::VecECP1;
use crate::presenter::view_model::buy_sell_signal::BuySellSignal;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
pub struct ECP1Analysis(pub Vec<BuySellSignal>);

impl From<VecECP1> for ECP1Analysis {
    fn from(value: VecECP1) -> Self {
        let vec_buy_sell_signal = value
            .0
            .into_iter()
            .map(|element| BuySellSignal {
                r#type: element.r#type,
                date: element.date,
            })
            .collect_vec();
        Self(vec_buy_sell_signal)
    }
}
