use crate::domain::entity::sma::SMA;
use chrono::NaiveDate;
use itertools::Itertools;
use std::cmp::Ordering as Ord;

/// クロスの向き
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CrossDirectionType {
    Golden,
    Dead,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CrossDirection<const N_DAY: usize, const O_DAY: usize> {
    /// ゴールデンクロス・デッドクロスになっていない場合は、None
    pub cross_direction: Option<CrossDirectionType>,
}

struct OrderingPair<const N_DAY: usize, const O_DAY: usize> {
    yesterday: Ordering<N_DAY, O_DAY>,
    today: Ordering<N_DAY, O_DAY>,
}

impl<const N_DAY: usize, const O_DAY: usize> From<OrderingPair<N_DAY, O_DAY>>
    for CrossDirection<N_DAY, O_DAY>
{
    fn from(value: OrderingPair<N_DAY, O_DAY>) -> Self {
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

/// 単純移動平均値の大小関係
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Ordering<const N_DAY: usize, const O_DAY: usize> {
    /// 対象日が片方なかったなど比較出来なかった場合は、None
    pub ordering: Option<Ord>,
}

struct SMAPair<'a, const N_DAY: usize, const O_DAY: usize> {
    n_day: &'a SMA<N_DAY>,
    o_day: Option<&'a SMA<O_DAY>>,
}

impl<'a, const N_DAY: usize, const O_DAY: usize> From<SMAPair<'a, N_DAY, O_DAY>>
    for Ordering<N_DAY, O_DAY>
{
    fn from(value: SMAPair<'a, N_DAY, O_DAY>) -> Self {
        let SMAPair { n_day, o_day } = value;
        match (n_day, o_day) {
            (n_day, Some(o_day)) => {
                let ordering = n_day.value.partial_cmp(&o_day.value);
                Ordering::<N_DAY, O_DAY> { ordering }
            }
            (_, _) => Ordering::<N_DAY, O_DAY>::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Cross {
    pub date: NaiveDate,
    pub five_day: Option<f64>,
    pub twenty_five_day: Option<f64>,
    pub ordering_5_25: Ordering<5, 25>,
    pub cross_direction: CrossDirection<5, 25>,
}

pub struct SMAPairList<'a, const N_DAY: usize, const O_DAY: usize> {
    pub n_days: &'a [SMA<N_DAY>],
    pub o_days: &'a [SMA<O_DAY>],
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecCross(pub Vec<Cross>);

impl<'a> From<SMAPairList<'a, 5, 25>> for VecCross {
    ///
    /// # Examples
    /// ```ignore
    /// let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
    /// let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
    /// let VecSMA(twenty_five_days) = VecSMA::<TWENTY_FIVE_DAY>::from(stocks.as_slice());
    /// let pair_sma = PairSMA {
    ///     n_days: five_days.as_slice(),
    ///     o_days: twenty_five_days.as_slice(),
    /// };
    /// let VecCross(crosses) = VecCross::from(pair_sma);
    /// assert_eq!(crosses.len(), 241);
    /// ```
    fn from(value: SMAPairList<'a, 5, 25>) -> Self {
        let SMAPairList {
            n_days: five_days,
            o_days: twenty_five_days,
        } = value;
        let crosses = five_days
            .iter()
            .map(|five_day| {
                let twenty_five_day = twenty_five_days
                    .iter()
                    .find(|twenty_five_day| five_day.date.eq(&twenty_five_day.date));
                let sma_pair = SMAPair {
                    n_day: five_day,
                    o_day: twenty_five_day,
                };
                let ordering_5_25 = Ordering::from(sma_pair);
                Cross {
                    date: five_day.date,
                    five_day: Some(five_day.value),
                    twenty_five_day: twenty_five_day.map(|twenty_five_day| twenty_five_day.value),
                    ordering_5_25,
                    ..Default::default()
                }
            })
            .inspect(|cross| {
                dbg!(cross);
            })
            .collect_vec()
            .windows(2)
            .map(|x| {
                let yesterday = x[0].ordering_5_25;
                let today = x[1].ordering_5_25;
                let ordering_pair = OrderingPair { yesterday, today };
                let cross_direction = CrossDirection::from(ordering_pair);
                Cross {
                    date: x[1].date,
                    five_day: x[1].five_day,
                    twenty_five_day: x[1].twenty_five_day,
                    ordering_5_25: x[1].ordering_5_25,
                    cross_direction,
                }
            })
            .collect_vec();
        Self(crosses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::sma::VecSMA;
    use crate::domain::entity::stock::VecStock;
    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const FIVE_DAY: usize = 5;
    const TWENTY_FIVE_DAY: usize = 25;

    #[test]
    fn vec_cross_test() {
        let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
        let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
        let VecSMA(twenty_five_days) = VecSMA::<TWENTY_FIVE_DAY>::from(stocks.as_slice());
        let pair_sma = SMAPairList {
            n_days: five_days.as_slice(),
            o_days: twenty_five_days.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(pair_sma);
        assert_eq!(crosses.len(), 241);
    }
}
