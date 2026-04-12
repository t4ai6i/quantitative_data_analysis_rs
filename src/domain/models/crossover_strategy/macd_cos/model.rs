use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::macd::model::{MACDs, MACD};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;

/// MACD Crossover Strategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MacdCos {
    pub date: NaiveDate,
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
    pub crossover_pattern: CrossoverPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deref, DerefMut)]
pub struct MacdCoses(Vec<MacdCos>);

impl<const F: usize, const S: usize, const SG: usize> From<MACDs<F, S, SG>> for MacdCoses {
    fn from(value: MACDs<F, S, SG>) -> Self {
        let vec_macd_cos = value
            .as_slice()
            .par_windows(2)
            .map(|windows| {
                // 昨日と今日のMACDとシグナルの比較からクロスオーバーパターンを判定
                let yesterday = windows[0].macd.partial_cmp(&windows[0].signal);
                let target = windows[1].macd.partial_cmp(&windows[1].signal);
                let crossover_pattern = CrossoverPattern::from((yesterday, target));
                let MACD {
                    date,
                    macd,
                    signal,
                    histogram,
                } = windows[1];
                MacdCos {
                    date,
                    macd,
                    signal,
                    histogram,
                    crossover_pattern,
                }
            })
            .collect::<Vec<MacdCos>>();
        Self(vec_macd_cos)
    }
}
