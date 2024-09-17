use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// 最新のECP2発生日・MSESP発生日・Cross発生日のセット
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CandleStickPatternDateSet {
    pub cross_date: NaiveDate,
    pub ecp2_date: NaiveDate,
    pub msesp_date: NaiveDate,
}
