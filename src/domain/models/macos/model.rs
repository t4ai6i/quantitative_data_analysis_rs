use crate::domain::models::sma::model::{SMAListPair, SMAPair, SMASet};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum::Display;

/// MovingAverageCrossoverStrategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOS {
    pub date: NaiveDate,
    pub sma_set_25: Option<SMASet<25>>,
    pub pattern_close_volume: PatternCloseVolume,
}

/// 終値・取引高における各移動平均線が交わったときの向きのパターンのセット
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct PatternCloseVolume {
    /// 終値ベースのMACOSのパターン
    pub close: Pattern,
    /// 出来高ベースのMACOSのパターン
    pub volume: Pattern,
}

/// 各移動平均線が交わったときの向きのパターン
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum Pattern {
    #[default]
    Neither,
    /// GoldenCross
    GoldenCross,
    /// DeadCross
    DeadCross,
}

/// Creates a `Pattern` from a tuple of two `Option<Ordering>` values.
///
/// # Parameters
/// - `value: (Option<Ordering>, Option<Ordering>)`  
///   A tuple containing two optional `Ordering` values, which represent comparisons.
///
/// # Returns
/// - `Pattern`:  
///   - Returns `Pattern::Golden` if the first value is `Some(Ordering::Less)` and the second is `Some(Ordering::Greater)`.
///   - Returns `Pattern::Dead` if the first value is `Some(Ordering::Greater)` and the second is `Some(Ordering::Less)`.
///   - Returns `Pattern::Neither` for all other cases.
///
/// # Behavior
/// - The function pattern matches on the provided tuple of `Option<Ordering>` values to determine which `Pattern` variant to return.
///
/// # Notes
/// - Provides a convenient way to map comparison results (`Ordering`) into a `Pattern` variant.
impl From<(Option<Ordering>, Option<Ordering>)> for Pattern {
    fn from(value: (Option<Ordering>, Option<Ordering>)) -> Self {
        match value {
            (Some(Ordering::Less), Some(Ordering::Greater)) => Self::GoldenCross,
            (Some(Ordering::Greater), Some(Ordering::Less)) => Self::DeadCross,
            _ => Self::Neither,
        }
    }
}

/// MACOS analysis pattern
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum AnalysisPattern {
    #[default]
    None,
    GoldenChance,
    DeadChance,
    GoldenLoss,
    DeadLoss,
}

/// A pair of `Pattern` and `rate_of_change` values.
pub(crate) struct PatternRateOfChangePair {
    pub pattern: Pattern,
    pub rate_of_change: f64,
}

/// Converts a `PatternRateOfChangePair` into an `AnalysisPattern`.
///
/// # Parameters
/// - `value: PatternRateOfChangePair`  
///   The input object containing a `Pattern` (either `Golden` or `Dead`) and a `rate_of_change` value for evaluation.
///
/// # Returns
/// - `AnalysisPattern`:  
///   - Returns `AnalysisPattern::GoldenChance` if the `Pattern` is `Golden` and the `rate_of_change` is greater than `0.0`.
///   - Returns `AnalysisPattern::DeadChance` if the `Pattern` is `Dead` and the `rate_of_change` is less than `0.0`.
///   - Returns `AnalysisPattern::GoldenLoss` if the `Pattern` is `Golden` and the `rate_of_change` is less than or equal to `0.0`.
///   - Returns `AnalysisPattern::DeadLoss` if the `Pattern` is `Dead` and the `rate_of_change` is greater than or equal to `0.0`.
///   - Returns `AnalysisPattern::None` if none of the above conditions are met.
///
/// # Behavior
/// - The conversion logic evaluates both the `Pattern` and the `rate_of_change` value
///   based on specific conditions to determine the appropriate `AnalysisPattern` variant.
///
/// # Notes
/// - This implementation provides a structured way to classify input `PatternRateOfChangePair` values into `AnalysisPattern` variants,
///   reflecting specific conditions of the pattern and its associated rate of change.
impl From<PatternRateOfChangePair> for AnalysisPattern {
    fn from(value: PatternRateOfChangePair) -> Self {
        match value {
            // GoldenChance: GoldenPattern/rate_of_change > 0.0
            PatternRateOfChangePair {
                pattern: Pattern::GoldenCross,
                rate_of_change: change,
            } if change > 0.0 => AnalysisPattern::GoldenChance,
            // DeadChance: DeadPattern/rate_of_change < 0.0
            PatternRateOfChangePair {
                pattern: Pattern::DeadCross,
                rate_of_change: change,
            } if change < 0.0 => AnalysisPattern::DeadChance,
            // GoldenLoss: GoldenPattern/rate_of_change <= 0.0
            PatternRateOfChangePair {
                pattern: Pattern::GoldenCross,
                rate_of_change: change,
            } if change <= 0.0 => AnalysisPattern::GoldenLoss,
            // DeadLoss: DeadPattern/rate_of_change >= 0.0
            PatternRateOfChangePair {
                pattern: Pattern::DeadCross,
                rate_of_change: change,
            } if change >= 0.0 => AnalysisPattern::DeadLoss,
            _ => AnalysisPattern::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct MACOSes(Vec<MACOS>);

impl MACOSes {
    /// Retrieves the latest date based on the closing value that matches a specified pattern.
    ///
    /// # Parameters
    /// - `pattern: &Pattern`  
    ///    A reference to the `Pattern` object used to match against the `close` value
    ///    in the `pattern_close_volume` of each `MACOS` item.
    ///
    /// # Returns
    /// - `Option<NaiveDate>`:  
    ///   - Returns `Some(date)` if one or more `MACOS` objects match the given pattern, where `date` is the latest matching date.
    ///   - Returns `None` if no matches are found.
    ///
    /// # Process
    /// - The method uses parallel iteration (`par_iter`) for efficiency when traversing the `MACOSES` collection.
    /// - It filters `MACOS` objects where the `close` field of `pattern_close_volume` equals the given `Pattern`.
    /// - The filtered results are then sorted by the `date` field in ascending order.
    /// - Finally, the latest date (if any) is extracted and returned.
    ///
    /// # Notes
    /// - This method leverages the `rayon` library for parallel processing, ideal for handling large datasets.
    /// - Sorting is done using `itertools`'s `sorted_by` for a clear and concise sorting step.
    pub fn latest_based_on_close(&self, pattern: &Pattern) -> Option<NaiveDate> {
        self.par_iter()
            .filter_map(|macos| {
                if macos.pattern_close_volume.close.eq(pattern) {
                    Some(*macos)
                } else {
                    None
                }
            })
            .max_by(|a, b| Ord::cmp(&a.date, &b.date))
            .map(|x| x.date)
    }
}

struct Intermediate {
    date: NaiveDate,
    sma_set_25: Option<SMASet<25>>,
    ordering_close_volume_5_25: OrderingCloseVolume<5, 25>,
}

impl<'a> From<SMAListPair<'a, 5, 25>> for MACOSes {
    ///
    /// # Examples
    /// ```
    /// use rayon::prelude::*;
    /// use quantitative_data_analysis_rs::domain::models::sma::model::{SMAListPair, SMAs};
    /// use quantitative_data_analysis_rs::domain::models::macos::model::MACOSes;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::data_format::csv::Csv;
    ///
    /// const CSV_8473: &[u8] = include_bytes!("../../../../assets/8473.T.csv");
    ///
    /// let successes: Vec<_> = Csv::from_slice::<true>(CSV_8473)
    ///     .into_par_iter().map(|s| s.unwrap()).collect();
    /// let vec_stock: Vec<_> = Csv::from_deserialize(successes)
    ///     .into_par_iter().map(|s| s.unwrap()).collect();
    /// let smas_5 = SMAs::<5>::from(vec_stock.as_slice());
    /// let smas_25 = SMAs::<25>::from(vec_stock.as_slice());
    /// let sma_list_pair = SMAListPair {
    ///     smas_n: smas_5.as_slice(),
    ///     smas_o: smas_25.as_slice(),
    /// };
    /// let macoses = MACOSes::from(sma_list_pair);
    /// assert_eq!(macoses.len(), 241);
    /// ```
    fn from(value: SMAListPair<'a, 5, 25>) -> Self {
        let SMAListPair {
            smas_n: smas_5,
            smas_o: smas_25,
        } = value;
        let intermediates: Vec<Intermediate> = smas_5
            .par_iter()
            .map(|sma_5| {
                let sma_25 = smas_25
                    .par_iter()
                    .find_first(|sma_25| sma_5.date.eq(&sma_25.date));
                let sma_pair = SMAPair {
                    sma_n: sma_5,
                    sma_o: sma_25,
                };
                let ordering_close_volume_5_25 = OrderingCloseVolume::from(sma_pair);
                Intermediate {
                    date: sma_5.date,
                    sma_set_25: sma_25.map(|sma_25| sma_25.sma_n),
                    ordering_close_volume_5_25,
                }
            })
            .collect();
        let vec_macos = intermediates
            .par_windows(2)
            .map(|x| {
                let yesterday = x[0].ordering_close_volume_5_25;
                let today = x[1].ordering_close_volume_5_25;
                let ordering_close_volume_pair = OrderingCloseVolumePastFuture {
                    past: yesterday,
                    future: today,
                };
                let pattern_close_volume = PatternCloseVolume::from(ordering_close_volume_pair);
                MACOS {
                    date: x[1].date,
                    sma_set_25: x[1].sma_set_25,
                    pattern_close_volume,
                }
            })
            .collect::<Vec<MACOS>>();
        Self(vec_macos)
    }
}

/// 終値ベース、出来高ベースそれぞれの大小関係
/// 対象日が片方なかったなど比較出来なかった場合は、None
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct OrderingCloseVolume<const N: usize, const O: usize> {
    /// 終値ベースの比較値
    pub close: Option<Ordering>,
    /// 取引高ベースの比較値
    pub volume: Option<Ordering>,
}

impl<'a, const N: usize, const O: usize> From<SMAPair<'a, N, O>> for OrderingCloseVolume<N, O> {
    fn from(value: SMAPair<'a, N, O>) -> Self {
        let SMAPair { sma_n, sma_o } = value;
        match (sma_n, sma_o) {
            (sma_n, Some(sma_o)) => {
                let close = sma_n.sma_n.close.partial_cmp(&sma_o.sma_n.close);
                let volume = sma_n.sma_n.volume.partial_cmp(&sma_o.sma_n.volume);
                OrderingCloseVolume::<N, O> { close, volume }
            }
            (_, _) => OrderingCloseVolume::<N, O>::default(),
        }
    }
}

struct OrderingCloseVolumePastFuture<const N: usize, const O: usize> {
    past: OrderingCloseVolume<N, O>,
    future: OrderingCloseVolume<N, O>,
}

impl<const N: usize, const O: usize> From<OrderingCloseVolumePastFuture<N, O>>
    for PatternCloseVolume
{
    fn from(value: OrderingCloseVolumePastFuture<N, O>) -> Self {
        // 前日と対象日の大小関係を比較して、ゴールデンクロスかデッドクロスかどちらも発生していないかを判定していく。
        // https://myfrankblog.com/find_golden_cross_and_dead_cross_by_python/#i-4
        let OrderingCloseVolumePastFuture { past, future } = value;
        let close = Pattern::from((past.close, future.close));
        let volume = Pattern::from((past.volume, future.volume));
        PatternCloseVolume { close, volume }
    }
}
