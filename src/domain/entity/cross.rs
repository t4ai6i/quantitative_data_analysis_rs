use std::cmp::Ordering as Ord;

use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::domain::entity::ordering::{Ordering, OrderingPair};
use crate::domain::entity::sma::{Average, SMAListPair, SMAPair};

/// 移動平均線が交わったときの向き
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum CrossDirectionType {
    /// ゴールデンクロス
    Golden,
    /// デッドクロス
    Dead,
    #[default]
    /// 上記どちらでもない場合
    Neither,
}

impl From<(Option<Ord>, Option<Ord>)> for CrossDirectionType {
    fn from(value: (Option<Ord>, Option<Ord>)) -> Self {
        match value {
            (Some(Ord::Less), Some(Ord::Greater)) => CrossDirectionType::Golden,
            (Some(Ord::Greater), Some(Ord::Less)) => CrossDirectionType::Dead,
            _ => CrossDirectionType::Neither,
        }
    }
}

/// 移動平均線NとOが交わったときの向き
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CrossDirection {
    pub close_average: CrossDirectionType,
    pub volume_average: CrossDirectionType,
}

impl<const N: usize, const O: usize> From<OrderingPair<N, O>> for CrossDirection {
    fn from(value: OrderingPair<N, O>) -> Self {
        // 前日と対象日の大小関係を比較して、ゴールデンクロスかデッドクロスかどちらも発生していないかを判定していく。
        // https://myfrankblog.com/find_golden_cross_and_dead_cross_by_python/#i-4
        let OrderingPair { past, future } = value;
        let close_average = CrossDirectionType::from((past.close_average, future.close_average));
        let volume_average = CrossDirectionType::from((past.volume_average, future.volume_average));
        CrossDirection {
            close_average,
            volume_average,
        }
    }
}

/// 5日移動平均線と25日移動平均線の交わりの情報
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Cross {
    pub date: NaiveDate,
    pub sma_5_average: Option<Average<5>>,
    pub sma_25_average: Option<Average<25>>,
    pub ordering_5_25: Ordering<5, 25>,
    pub cross_direction_5_25: CrossDirection,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecCross(pub Vec<Cross>);

impl<'a> From<SMAListPair<'a, 5, 25>> for VecCross {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::cross::VecCross;
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
    /// let VecCross(crosses) = VecCross::from(sma_list_pair);
    /// assert_eq!(crosses.len(), 241);
    /// ```
    fn from(value: SMAListPair<'a, 5, 25>) -> Self {
        let SMAListPair {
            smas_n: smas_5,
            smas_o: smas_25,
        } = value;
        let crosses = smas_5
            .iter()
            .map(|sma_5| {
                let sma_25 = smas_25.iter().find(|sma_25| sma_5.date.eq(&sma_25.date));
                let sma_pair = SMAPair {
                    sma_n: sma_5,
                    sma_o: sma_25,
                };
                let ordering_5_25 = Ordering::from(sma_pair);
                Cross {
                    date: sma_5.date,
                    sma_5_average: Some(sma_5.average),
                    sma_25_average: sma_25.map(|sma_25| sma_25.average),
                    ordering_5_25,
                    ..Default::default()
                }
            })
            .collect_vec()
            .windows(2)
            .map(|x| {
                let yesterday = x[0].ordering_5_25;
                let today = x[1].ordering_5_25;
                let ordering_pair = OrderingPair {
                    past: yesterday,
                    future: today,
                };
                let cross_direction_5_25 = CrossDirection::from(ordering_pair);
                Cross {
                    date: x[1].date,
                    sma_5_average: x[1].sma_5_average,
                    sma_25_average: x[1].sma_25_average,
                    ordering_5_25: x[1].ordering_5_25,
                    cross_direction_5_25,
                }
            })
            .collect_vec();
        Self(crosses)
    }
}
