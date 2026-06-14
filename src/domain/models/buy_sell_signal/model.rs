use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::Display;

/// 売買シグナルタイプ
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum BuySellSignalType {
    #[default]
    Stay,
    Buy,
    Sell,
}

impl From<CrossoverPattern> for BuySellSignalType {
    fn from(crossover_pattern: CrossoverPattern) -> Self {
        match crossover_pattern {
            CrossoverPattern::Neither => Self::Stay,
            CrossoverPattern::GoldenCross => Self::Buy,
            CrossoverPattern::DeadCross => Self::Sell,
        }
    }
}

/// 売買シグナル
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BuySellSignal {
    pub r#type: BuySellSignalType,
    pub date: NaiveDate,
}

#[cfg(test)]
pub(crate) mod tests {
    use rayon::prelude::*;

    use crate::domain::models::buy_sell_signal::model::BuySellSignal;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Stay};

    pub(crate) struct TupleVecBuySellSignal(pub(crate) (Vec<BuySellSignal>, Vec<BuySellSignal>));

    impl From<&[BuySellSignal]> for TupleVecBuySellSignal {
        fn from(value: &[BuySellSignal]) -> Self {
            let tuple: (Vec<_>, Vec<_>) = value
                .par_iter()
                .filter(|s| s.r#type.ne(&Stay))
                .partition(|signal| signal.r#type.eq(&Buy));
            TupleVecBuySellSignal(tuple)
        }
    }
}
