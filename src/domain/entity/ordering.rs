use crate::domain::entity::sma::SMAPair;
use std::cmp;

/// 値の大小関係
/// 対象日が片方なかったなど比較出来なかった場合は、None
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Ordering<const N: usize, const O: usize>(pub Option<cmp::Ordering>);

impl<'a, const N: usize, const O: usize> From<SMAPair<'a, N, O>> for Ordering<N, O> {
    fn from(value: SMAPair<'a, N, O>) -> Self {
        let SMAPair { sma_n, sma_o } = value;
        match (sma_n, sma_o) {
            (sma_n, Some(sma_o)) => {
                let ordering = sma_n.ave.partial_cmp(&sma_o.ave);
                Ordering::<N, O>(ordering)
            }
            (_, _) => Ordering::<N, O>::default(),
        }
    }
}

pub struct OrderingPair<const N: usize, const O: usize> {
    pub past: Ordering<N, O>,
    pub future: Ordering<N, O>,
}
