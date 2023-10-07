use crate::domain::entity::sma::SMA;
use chrono::NaiveDate;
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CrossDirection {
    Golden,
    Dead,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Comp<const N_DAY: usize, const O_DAY: usize> {
    /// クロスの向き
    /// ゴールデンクロス・デッドクロスになっていない場合は、None
    pub cross_direction: Option<CrossDirection>,
    /// 移動平均の値の大小関係
    /// 対象日が片方なかったなど、比較出来なかった場合は、None
    pub comp: Option<Ordering>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Cross {
    pub date: NaiveDate,
    pub five_day: Option<f64>,
    pub twenty_five_day: Option<f64>,
    pub comp_5_25: Comp<5, 25>,
}

pub struct PairSMA<'a, const N_DAY: usize, const O_DAY: usize> {
    pub pair1: &'a [SMA<N_DAY>],
    pub pair2: &'a [SMA<O_DAY>],
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecCross(pub Vec<Cross>);

impl<'a, const N_DAY: usize, const O_DAY: usize> From<PairSMA<'a, N_DAY, O_DAY>> for VecCross {
    fn from(value: PairSMA<'a, N_DAY, O_DAY>) -> Self {
        let PairSMA { pair1, pair2 } = value;
        let crosses = pair1
            .iter()
            .map(|five_day| {
                let twenty_five_day_found = pair2
                    .iter()
                    .find(|twenty_five_day| five_day.date.eq(&twenty_five_day.date));
                // TODO: five_dayとtwenty_five_dayの値の大小関係を構造体メンバに記録する。
                // TODO: 前日の大小関係と対象日の大小関係を比較して、ゴールデンクロスかデッドクロスかどちらのクロスも発生していないかを判定していく。
                // TODO: GoldenCross発生当日の調整後終値と比べ、3営業日後の値が上昇したか判定していく。
                Cross {
                    date: five_day.date,
                    five_day: Some(five_day.value),
                    twenty_five_day: twenty_five_day_found
                        .map(|twenty_five_day| twenty_five_day.value),
                    ..Default::default()
                }
            })
            .inspect(|cross| {
                dbg!(cross);
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
    fn vecsma_test() {
        let VecStock(stocks) = VecStock::<true>::from(CSV_8473);
        let VecSMA(five_days) = VecSMA::<FIVE_DAY>::from(stocks.as_slice());
        let VecSMA(twenty_five_days) = VecSMA::<TWENTY_FIVE_DAY>::from(stocks.as_slice());
        let pair_sma = PairSMA {
            pair1: five_days.as_slice(),
            pair2: twenty_five_days.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(pair_sma);
        assert_eq!(crosses.len(), 242);
    }
}
