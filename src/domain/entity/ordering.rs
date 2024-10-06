use crate::domain::entity::sma::SMAPair;
use std::cmp::Ordering as Ord;

/// 値の大小関係
/// 対象日が片方なかったなど比較出来なかった場合は、None
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Ordering<const N: usize, const O: usize> {
    /// 終値平均の比較値
    pub close: Option<Ord>,
    /// 取引高平均の比較値
    pub volume: Option<Ord>,
}

impl<'a, const N: usize, const O: usize> From<SMAPair<'a, N, O>> for Ordering<N, O> {
    fn from(value: SMAPair<'a, N, O>) -> Self {
        let SMAPair { sma_n, sma_o } = value;
        match (sma_n, sma_o) {
            (sma_n, Some(sma_o)) => {
                let close = sma_n.sma_n.close.partial_cmp(&sma_o.sma_n.close);
                let volume = sma_n.sma_n.volume.partial_cmp(&sma_o.sma_n.volume);
                Ordering::<N, O> { close, volume }
            }
            (_, _) => Ordering::<N, O>::default(),
        }
    }
}

pub struct OrderingPair<const N: usize, const O: usize> {
    pub past: Ordering<N, O>,
    pub future: Ordering<N, O>,
}
