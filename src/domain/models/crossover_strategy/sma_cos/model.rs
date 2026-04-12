use crate::domain::models::crossover_strategy::crossover_pattern;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::sma::model::{SMAListPair, SMAPair};
use crate::domain::models::{close, volume};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use std::cmp::Ordering;

/// SimpleMovingAverageCrossoverStrategy
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SmaCos {
    pub date: NaiveDate,
    pub close_25: Option<close::model::F64>,
    pub volume_25: Option<volume::model::F64>,
    pub crossover_pattern_close: crossover_pattern::close::model::CrossoverPattern,
    pub crossover_pattern_volume: crossover_pattern::volume::model::CrossoverPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct SmaCoses(Vec<SmaCos>);

impl SmaCoses {
    /// Retrieves the latest date based on the closing value that matches a specified pattern.
    ///
    /// # Parameters
    /// - `crossover_pattern: &CrossoverPattern`
    ///   A reference to the `CrossoverPattern` object used to match against the `close` value
    ///   in the `pattern_close_volume` of each `SmaCos` item.
    ///
    /// # Returns
    /// - `Option<NaiveDate>`:  
    ///   - Returns `Some(date)` if one or more `SmaCos` objects match the given pattern, where `date` is the latest matching date.
    ///   - Returns `None` if no matches are found.
    ///
    /// # Process
    /// - The method uses parallel iteration (`par_iter`) for efficiency when traversing the `MACOSES` collection.
    /// - It filters `SmaCos` objects where the `close` field of `pattern_close_volume` equals the given `CrossoverPattern`.
    /// - The filtered results are then sorted by the `date` field in ascending order.
    /// - Finally, the latest date (if any) is extracted and returned.
    ///
    /// # Notes
    /// - This method leverages the `rayon` library for parallel processing, ideal for handling large datasets.
    /// - Sorting is done using `itertools`'s `sorted_by` for a clear and concise sorting step.
    pub fn latest_based_on_close(&self, crossover_pattern: &CrossoverPattern) -> Option<NaiveDate> {
        self.par_iter()
            .filter_map(|sma_cos| {
                if sma_cos.crossover_pattern_close.0.eq(crossover_pattern) {
                    Some(sma_cos)
                } else {
                    None
                }
            })
            .max_by(|a, b| Ord::cmp(&a.date, &b.date))
            .map(|x| x.date)
    }
}

impl<'a> From<SMAListPair<'a, 5, 25>> for SmaCoses {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    /// use rayon::prelude::*;
    ///
    /// use quantitative_data_analysis_rs::domain::models::sma::model::{SMAListPair, SMAs};
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const CSV: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let smas_5 = SMAs::<5>::from(stocks.as_slice());
    ///   let smas_25 = SMAs::<25>::from(stocks.as_slice());
    ///   let sma_list_pair = SMAListPair {
    ///       smas_n: smas_5.as_slice(),
    ///       smas_o: smas_25.as_slice(),
    ///   };
    ///   let sma_coses = SmaCoses::from(sma_list_pair);
    ///   assert_eq!(sma_coses.len(), 241);
    /// });
    /// ```
    fn from(value: SMAListPair<'a, 5, 25>) -> Self {
        let SMAListPair {
            smas_n: smas_5,
            smas_o: smas_25,
        } = value;
        let vec_sma_cos = smas_5
            .par_iter()
            .map(|sma_5| {
                let sma_25 = smas_25
                    .par_iter()
                    .find_first(|sma_25| sma_25.date.eq(&sma_5.date));
                let sma_pair = SMAPair {
                    sma_n: sma_5,
                    sma_o: sma_25,
                };
                let ordering_close_volume = OrderingCloseVolume::from(sma_pair);
                let (close, volume) = match sma_25 {
                    Some(sma_25) => (Some(sma_25.close), Some(sma_25.volume)),
                    None => (None, None),
                };
                (sma_5.date, close, volume, ordering_close_volume)
            })
            .collect::<Vec<(
                NaiveDate,
                Option<close::model::F64>,
                Option<volume::model::F64>,
                OrderingCloseVolume,
            )>>()
            .par_windows(2)
            .map(|windows| {
                let &(_, _, _, yesterday_ordering_close_volume) = &windows[0];
                let &(target, target_close, target_volume, target_ordering_close_volume) =
                    &windows[1];
                let ordering_close_volume_pair = OrderingCloseVolumePair {
                    past: yesterday_ordering_close_volume,
                    target: target_ordering_close_volume,
                };
                let CrossoverPatternCloseVolume {
                    close: crossover_pattern_close,
                    volume: crossover_pattern_volume,
                } = CrossoverPatternCloseVolume::from(ordering_close_volume_pair);
                SmaCos {
                    date: target,
                    close_25: target_close,
                    volume_25: target_volume,
                    crossover_pattern_close: crossover_pattern::close::model::CrossoverPattern(
                        crossover_pattern_close,
                    ),
                    crossover_pattern_volume: crossover_pattern::volume::model::CrossoverPattern(
                        crossover_pattern_volume,
                    ),
                }
            })
            .collect::<Vec<SmaCos>>();
        Self(vec_sma_cos)
    }
}

/// 終値ベース、出来高ベースそれぞれの大小関係
/// 対象日が片方なかったなど比較出来なかった場合は、None
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
struct OrderingCloseVolume {
    /// 終値ベースの比較値
    pub close: Option<Ordering>,
    /// 取引高ベースの比較値
    pub volume: Option<Ordering>,
}

impl<'a, const N: usize, const O: usize> From<SMAPair<'a, N, O>> for OrderingCloseVolume {
    fn from(value: SMAPair<'a, N, O>) -> Self {
        let SMAPair { sma_n, sma_o } = value;
        match (sma_n, sma_o) {
            (sma_n, Some(sma_o)) => {
                let close = sma_n.close.0.partial_cmp(&sma_o.close.0);
                let volume = sma_n.volume.0.partial_cmp(&sma_o.volume.0);
                OrderingCloseVolume { close, volume }
            }
            (_, _) => OrderingCloseVolume::default(),
        }
    }
}

struct OrderingCloseVolumePair {
    pub past: OrderingCloseVolume,
    pub target: OrderingCloseVolume,
}

/// 終値・取引高における各移動平均線が交わったときの向きのパターンのセット
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CrossoverPatternCloseVolume {
    /// 終値ベースのMACOSのパターン
    pub close: CrossoverPattern,
    /// 出来高ベースのMACOSのパターン
    pub volume: CrossoverPattern,
}

impl From<OrderingCloseVolumePair> for CrossoverPatternCloseVolume {
    fn from(value: OrderingCloseVolumePair) -> Self {
        // 前日と対象日の大小関係を比較して、ゴールデンクロスかデッドクロスかどちらも発生していないかを判定していく。
        // https://myfrankblog.com/find_golden_cross_and_dead_cross_by_python/#i-4
        let OrderingCloseVolumePair { past, target } = value;
        let close = CrossoverPattern::from((past.close, target.close));
        let volume = CrossoverPattern::from((past.volume, target.volume));
        CrossoverPatternCloseVolume { close, volume }
    }
}
