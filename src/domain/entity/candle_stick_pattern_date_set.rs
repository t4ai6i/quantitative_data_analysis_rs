use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 最新のECP2発生日・MSESP発生日・MACOS発生日・MACPS発生日のセット
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CandleStickPatternDateSet {
    pub macos_date: NaiveDate,
    pub ecp2_date: NaiveDate,
    pub msesp_date: NaiveDate,
    pub macps_date: NaiveDate,
}
