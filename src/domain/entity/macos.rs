use rayon::prelude::*;

use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::domain::entity::ordering::{Ordering, OrderingPair};
use crate::domain::entity::sma::{SMAListPair, SMAPair, SMASet};

/// 各移動平均線が交わったときの向きタイプ
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum MACOSType {
    /// ゴールデンクロス
    Golden,
    /// デッドクロス
    Dead,
    #[default]
    /// どちらでもない場合
    Neither,
}

impl From<(Option<std::cmp::Ordering>, Option<std::cmp::Ordering>)> for MACOSType {
    fn from(value: (Option<std::cmp::Ordering>, Option<std::cmp::Ordering>)) -> Self {
        match value {
            (Some(std::cmp::Ordering::Less), Some(std::cmp::Ordering::Greater)) => {
                MACOSType::Golden
            }
            (Some(std::cmp::Ordering::Greater), Some(std::cmp::Ordering::Less)) => MACOSType::Dead,
            _ => MACOSType::Neither,
        }
    }
}

/// 終値・取引高における各移動平均線が交わったときの向きタイプの集合
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct MACOSSet {
    /// 終値ベースのMovingAverageCrossoverStrategy
    pub close: MACOSType,
    /// 出来高ベースのMovingAverageCrossoverStrategy
    pub volume: MACOSType,
}

impl<const N: usize, const O: usize> From<OrderingPair<N, O>> for MACOSSet {
    fn from(value: OrderingPair<N, O>) -> Self {
        // 前日と対象日の大小関係を比較して、ゴールデンクロスかデッドクロスかどちらも発生していないかを判定していく。
        // https://myfrankblog.com/find_golden_cross_and_dead_cross_by_python/#i-4
        let OrderingPair { past, future } = value;
        let close = MACOSType::from((past.close, future.close));
        let volume = MACOSType::from((past.volume, future.volume));
        MACOSSet { close, volume }
    }
}

/// MovingAverageCrossoverStrategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOS {
    pub date: NaiveDate,
    pub sma_set_25: Option<SMASet<25>>,
    pub macos_set_5_25: MACOSSet,
}

struct MACOSIntermediate {
    date: NaiveDate,
    sma_set_25: Option<SMASet<25>>,
    ordering_5_25: Ordering<5, 25>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecMACOS(pub Vec<MACOS>);

impl VecMACOS {
    pub fn latest_based_on_close(&self, r#type: &MACOSType) -> Option<NaiveDate> {
        let filtered: Vec<MACOS> = self
            .0
            .par_iter()
            .filter_map(|macos| {
                if macos.macos_set_5_25.close.eq(r#type) {
                    Some(*macos)
                } else {
                    None
                }
            })
            .collect();
        filtered
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .last()
            .map(|x| x.date)
    }
}

impl<'a> From<SMAListPair<'a, 5, 25>> for VecMACOS {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::macos::VecMACOS;
    /// use quantitative_data_analysis_rs::domain::entity::sma::{SMAListPair, VecSMA};
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    ///
    /// let vec_stock = Csv::from_slice::<true>(CSV_8473);
    /// let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
    /// let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
    /// let sma_list_pair = SMAListPair {
    ///     smas_n: smas_5.as_slice(),
    ///     smas_o: smas_25.as_slice(),
    /// };
    /// let VecMACOS(macoses) = VecMACOS::from(sma_list_pair);
    /// assert_eq!(macoses.len(), 241);
    /// ```
    fn from(value: SMAListPair<'a, 5, 25>) -> Self {
        let SMAListPair {
            smas_n: smas_5,
            smas_o: smas_25,
        } = value;
        let macoses: Vec<MACOSIntermediate> = smas_5
            .par_iter()
            .map(|sma_5| {
                let sma_25 = smas_25
                    .par_iter()
                    .find_first(|sma_25| sma_5.date.eq(&sma_25.date));
                let sma_pair = SMAPair {
                    sma_n: sma_5,
                    sma_o: sma_25,
                };
                let ordering_5_25 = Ordering::from(sma_pair);
                MACOSIntermediate {
                    date: sma_5.date,
                    sma_set_25: sma_25.map(|sma_25| sma_25.sma_n),
                    ordering_5_25,
                }
            })
            .collect();
        let macoses = macoses
            .windows(2)
            .map(|x| {
                let yesterday = x[0].ordering_5_25;
                let today = x[1].ordering_5_25;
                let ordering_pair = OrderingPair {
                    past: yesterday,
                    future: today,
                };
                let macos_set_5_25 = MACOSSet::from(ordering_pair);
                MACOS {
                    date: x[1].date,
                    sma_set_25: x[1].sma_set_25,
                    macos_set_5_25,
                }
            })
            .collect_vec();
        Self(macoses)
    }
}
