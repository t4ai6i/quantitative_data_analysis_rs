use crate::domain::models::close;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{
    DerivedFact, DirectionType, EventFact, EventKind, EventParams, MetricKind,
};
use crate::domain::models::volume;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use ta::Next;
use ta::indicators::SimpleMovingAverage;

/// 終値、取引高の単純移動平均
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA<const N: usize> {
    /// N日目の日付
    pub date: NaiveDate,
    /// 終値のN日単純移動平均
    pub close: close::model::F64,
    /// 取引高のN日単純移動平均
    pub volume: volume::model::F64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct SMAs<const N: usize>(Vec<SMA<N>>);

impl<const N: usize> From<&[Stock]> for SMAs<N> {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    /// use rayon::prelude::*;
    ///
    /// use quantitative_data_analysis_rs::domain::models::sma::model::SMAs;
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const DAYS_5: usize = 5;
    /// const DAYS_25: usize = 25;
    /// const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let smas_5 = SMAs::<DAYS_5>::from(stocks.as_slice());
    ///   assert_eq!(smas_5.len(), 242);
    ///
    ///   let smas_25 = SMAs::<DAYS_25>::from(stocks.as_slice());
    ///   assert_eq!(smas_25.len(), 222);
    ///
    ///   let stocks: Vec<model::Stock> = vec![];
    ///   let smas_5 = SMAs::<DAYS_5>::from(stocks.as_slice());
    ///   assert_eq!(smas_5.len(), 0);
    /// });
    ///
    /// ```
    fn from(value: &[Stock]) -> Self {
        let vec_sma = value
            .par_windows(N)
            .map(|stocks| {
                // 終値のN日と取引高のN日の単純移動平均
                let mut sma_close = SimpleMovingAverage::new(N).unwrap();
                let mut sma_volume = SimpleMovingAverage::new(N).unwrap();
                let mut close = 0.0;
                let mut volume = 0.0;
                for stock in stocks {
                    close = sma_close.next(stock.close);
                    volume = sma_volume.next(stock.volume as f64);
                }
                SMA {
                    date: stocks.last().unwrap().date,
                    close: close::model::F64(close),
                    volume: volume::model::F64(volume),
                }
            })
            .collect::<Vec<SMA<N>>>();
        Self(vec_sma)
    }
}

impl<const N: usize> From<&SMAs<N>> for Vec<DerivedFact> {
    fn from(value: &SMAs<N>) -> Self {
        value
            .iter()
            .map(|sma| DerivedFact {
                metric: MetricKind::Sma,
                date: sma.date,
                value: sma.close.0,
                period: Some(N as u32),
            })
            .collect()
    }
}

pub struct SmaCrossEvents<'a, const N: usize, const O: usize> {
    pub sma_list_pair: SMAListPair<'a, N, O>,
}

impl<'a, const N: usize, const O: usize> From<SmaCrossEvents<'a, N, O>> for Vec<EventFact> {
    fn from(value: SmaCrossEvents<'a, N, O>) -> Self {
        let SmaCrossEvents {
            sma_list_pair: SMAListPair { smas_n, smas_o },
        } = value;

        smas_n
            .windows(2)
            .filter_map(|window| {
                let past = &window[0];
                let target = &window[1];

                let past_sma_o = smas_o.iter().find(|sma| sma.date == past.date)?;
                let target_sma_o = smas_o.iter().find(|sma| sma.date == target.date)?;

                let pattern = CrossoverPattern::from((
                    past.close.0.partial_cmp(&past_sma_o.close.0),
                    target.close.0.partial_cmp(&target_sma_o.close.0),
                ));

                match pattern {
                    CrossoverPattern::GoldenCross => Some(EventFact {
                        kind: EventKind::GoldenCross,
                        occurred_at: target.date,
                        direction: DirectionType::Uptrend,
                        event_params: EventParams::Cross {
                            fast_metric: MetricKind::Sma,
                            fast_period: N as u32,
                            slow_metric: MetricKind::Sma,
                            slow_period: O as u32,
                        },
                    }),
                    CrossoverPattern::DeadCross => Some(EventFact {
                        kind: EventKind::DeadCross,
                        occurred_at: target.date,
                        direction: DirectionType::Downtrend,
                        event_params: EventParams::Cross {
                            fast_metric: MetricKind::Sma,
                            fast_period: N as u32,
                            slow_metric: MetricKind::Sma,
                            slow_period: O as u32,
                        },
                    }),
                    CrossoverPattern::Neither => None,
                }
            })
            .collect()
    }
}

pub struct SMAPair<'a, const N: usize, const O: usize> {
    pub sma_n: &'a SMA<N>,
    pub sma_o: Option<&'a SMA<O>>,
}

pub struct SMAListPair<'a, const N: usize, const O: usize> {
    pub smas_n: &'a [SMA<N>],
    pub smas_o: &'a [SMA<O>],
}

pub struct SMAListTrio<'a, const N: usize, const O: usize, const P: usize> {
    pub smas_n: &'a [SMA<N>],
    pub smas_o: &'a [SMA<O>],
    pub smas_p: &'a [SMA<P>],
}

#[cfg(test)]
mod tests {
    use super::{SMAListPair, SMAs, SmaCrossEvents};
    use crate::domain::models::stock::model::Stock;
    use crate::domain::models::technical_analysis::model::{
        DerivedFact, EventFact, EventKind, MetricKind,
    };
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    fn stock(date: NaiveDate, close: f64, volume: u64) -> Stock {
        Stock {
            date,
            close,
            volume,
            ..Default::default()
        }
    }

    #[test]
    fn converts_close_values_to_derived_facts() {
        let stocks = (0..6)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0 + i as f64,
                    1_000 + i as u64,
                )
            })
            .collect::<Vec<_>>();

        let smas = SMAs::<3>::from(stocks.as_slice());
        let facts: Vec<DerivedFact> = Vec::from(&smas);

        assert_eq!(facts.len(), 4);
        assert!(facts.iter().all(|fact| fact.metric == MetricKind::Sma));
        assert!(facts.iter().all(|fact| fact.period == Some(3)));
        assert_eq!(facts[0].date, NaiveDate::from_ymd_opt(2024, 1, 3).unwrap());
        assert_eq!(
            facts.last().unwrap().date,
            NaiveDate::from_ymd_opt(2024, 1, 6).unwrap()
        );
        assert!((facts.last().unwrap().value - 104.0).abs() < 1e-9);
    }

    #[test]
    fn converts_sma_5_25_crossovers_to_events() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let stocks = (0..60)
            .map(|i| {
                let close = if i < 25 {
                    100.0 - i as f64
                } else {
                    10.0 + (i - 25) as f64 * 2.0
                };
                let date = start_date + chrono::Duration::days(i as i64);
                stock(date, close, 1_000 + i as u64)
            })
            .collect::<Vec<_>>();

        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());
        let events: Vec<EventFact> = Vec::from(SmaCrossEvents {
            sma_list_pair: SMAListPair {
                smas_n: smas_5.as_slice(),
                smas_o: smas_25.as_slice(),
            },
        });

        assert!(
            events
                .iter()
                .any(|event| event.kind == EventKind::GoldenCross)
        );
    }
}
