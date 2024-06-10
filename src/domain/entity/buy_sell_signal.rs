use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use strum::Display;

/// 売買シグナルタイプ
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
#[serde(tag = "kind")]
pub enum BuySellSignalType {
    #[default]
    Stay,
    Buy,
    Sell,
}

/// 売買シグナル
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BuySellSignal {
    pub r#type: BuySellSignalType,
    pub date: NaiveDate,
}
