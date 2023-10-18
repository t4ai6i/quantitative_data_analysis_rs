use crate::domain::entity::sma::SMA;
use charts_rs::NIL_VALUE;
use chrono::NaiveDate;
use itertools::Itertools;
use std::cmp::Ordering as Ord;

/// 単純移動平均値の大小関係
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Ordering<const N: usize, const O: usize> {
    /// 対象日が片方なかったなど比較出来なかった場合は、None
    pub ordering: Option<Ord>,
}

struct SMAPair<'a, const N: usize, const O: usize> {
    sma_n: &'a SMA<N>,
    sma_o: Option<&'a SMA<O>>,
}

impl<'a, const N: usize, const O: usize> From<SMAPair<'a, N, O>> for Ordering<N, O> {
    fn from(value: SMAPair<'a, N, O>) -> Self {
        let SMAPair { sma_n, sma_o } = value;
        match (sma_n, sma_o) {
            (sma_n, Some(sma_o)) => {
                let ordering = sma_n.ave.partial_cmp(&sma_o.ave);
                Ordering::<N, O> { ordering }
            }
            (_, _) => Ordering::<N, O>::default(),
        }
    }
}

/// クロスの向き
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CrossDirectionType {
    Golden,
    Dead,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CrossDirection<const N: usize, const O: usize> {
    /// ゴールデンクロス・デッドクロスになっていない場合は、None
    pub cross_direction: Option<CrossDirectionType>,
}

struct OrderingPair<const N: usize, const O: usize> {
    yesterday: Ordering<N, O>,
    today: Ordering<N, O>,
}

impl<const N: usize, const O: usize> From<OrderingPair<N, O>> for CrossDirection<N, O> {
    fn from(value: OrderingPair<N, O>) -> Self {
        // 前日の大小関係と対象日の大小関係を比較して、ゴールデンクロスかデッドクロスかどちらも発生していないかを判定していく。
        // https://myfrankblog.com/find_golden_cross_and_dead_cross_by_python/#i-4
        let OrderingPair { yesterday, today } = value;
        match (yesterday.ordering, today.ordering) {
            (Some(Ord::Less), Some(Ord::Greater)) => CrossDirection {
                cross_direction: Some(CrossDirectionType::Golden),
            },
            (Some(Ord::Greater), Some(Ord::Less)) => CrossDirection {
                cross_direction: Some(CrossDirectionType::Dead),
            },
            _ => CrossDirection {
                cross_direction: None,
            },
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Cross {
    pub date: NaiveDate,
    pub sma_5_ave: Option<f64>,
    pub sma_25_ave: Option<f64>,
    pub ordering_5_25: Ordering<5, 25>,
    pub cross_direction_5_25: CrossDirection<5, 25>,
}

pub struct SMAListPair<'a, const N: usize, const O: usize> {
    pub smas_n: &'a [SMA<N>],
    pub smas_o: &'a [SMA<O>],
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecCross(pub Vec<Cross>);

impl<'a> From<SMAListPair<'a, 5, 25>> for VecCross {
    ///
    /// # Examples
    /// ```ignore
    /// let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
    /// let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
    /// let VecSMA(smas_25) = VecSMA::<25>::from(stocks.as_slice());
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
                    sma_5_ave: Some(sma_5.ave),
                    sma_25_ave: sma_25.map(|sma_25| sma_25.ave),
                    ordering_5_25,
                    ..Default::default()
                }
            })
            .collect_vec()
            .windows(2)
            .map(|x| {
                let yesterday = x[0].ordering_5_25;
                let today = x[1].ordering_5_25;
                let ordering_pair = OrderingPair { yesterday, today };
                let cross_direction_5_25 = CrossDirection::from(ordering_pair);
                Cross {
                    date: x[1].date,
                    sma_5_ave: x[1].sma_5_ave,
                    sma_25_ave: x[1].sma_25_ave,
                    ordering_5_25: x[1].ordering_5_25,
                    cross_direction_5_25,
                }
            })
            .collect_vec();
        Self(crosses)
    }
}

pub trait VecCrossExt {
    fn collect_vec_cross(&self, r#type: CrossDirectionType) -> Vec<f32>;
}

impl VecCrossExt for VecCross {
    fn collect_vec_cross(&self, r#type: CrossDirectionType) -> Vec<f32> {
        self.0
            .iter()
            .map(|cross| match cross.cross_direction_5_25 {
                CrossDirection { cross_direction }
                    if cross_direction.filter(|x| x.eq(&r#type)).is_some() =>
                {
                    cross.sma_25_ave.unwrap() as _
                }
                _ => NIL_VALUE,
            })
            .collect_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::sma::VecSMA;
    use crate::domain::entity::stock::VecStock;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn vec_cross_test() {
        let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
        let VecSMA(smas_25) = VecSMA::<25>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(sma_list_pair);
        assert_eq!(crosses.len(), 241);
    }
}
