use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{
    DerivedFact, EventFact, EventKind, EventParams, MetricKind,
};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use ta::Next;
use ta::indicators::MovingAverageConvergenceDivergence;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACD {
    pub date: NaiveDate,
    pub macd: f64,
    pub signal: f64,
    pub histogram: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct MACDs<const F: usize, const S: usize, const SG: usize>(Vec<MACD>);

/// # Examples
/// ```
/// use bytes::Bytes;
/// use quantitative_data_analysis_rs::domain::models::macd::model::MACDs;
/// use quantitative_data_analysis_rs::domain::models::stock::model;
/// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
/// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
/// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
/// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
///
/// const FAST_PERIOD: usize = 12;
/// const SLOW_PERIOD: usize = 26;
/// const SIGNAL_PERIOD: usize = 9;
/// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
///
/// tokio_test::block_on(async {
///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
///   let query = queries::get_stocks::Query {
///     ..Default::default()
///   };
///   let stocks = dsv.get_stocks(&query).await.unwrap();
///   let macds = MACDs::<FAST_PERIOD, SLOW_PERIOD, SIGNAL_PERIOD>::from(stocks.as_slice());
///   assert_eq!(macds.len(), 35);
/// });
/// ```
impl<const F: usize, const S: usize, const SG: usize> From<&[Stock]> for MACDs<F, S, SG> {
    fn from(value: &[Stock]) -> Self {
        const {
            assert!(F > 0, "F(fast period) is greater than 0");
        }
        const {
            assert!(S > 0, "S(slow period) is greater than 0");
        }
        const {
            assert!(SG > 0, "SG(signal period) is greater than 0");
        }
        let mut macd = MovingAverageConvergenceDivergence::new(F, S, SG).unwrap();
        let macds = value
            .iter()
            .map(|stock| {
                let (macd, signal, histogram) = macd.next(stock.close).into();
                MACD {
                    date: stock.date,
                    macd,
                    signal,
                    histogram,
                }
            })
            .collect::<Vec<MACD>>();
        MACDs(macds)
    }
}

impl<const F: usize, const S: usize, const SG: usize> From<&MACDs<F, S, SG>> for Vec<DerivedFact> {
    fn from(value: &MACDs<F, S, SG>) -> Self {
        value
            .iter()
            .flat_map(|macd| {
                [
                    DerivedFact {
                        metric: MetricKind::Macd,
                        date: macd.date,
                        value: macd.macd,
                        period: Some(S as u32),
                    },
                    DerivedFact {
                        metric: MetricKind::Signal,
                        date: macd.date,
                        value: macd.signal,
                        period: Some(SG as u32),
                    },
                ]
            })
            .collect()
    }
}

pub struct MacdCrossEvents<'a, const F: usize, const S: usize, const SG: usize> {
    pub macds: &'a MACDs<F, S, SG>,
}

impl<'a, const F: usize, const S: usize, const SG: usize> From<MacdCrossEvents<'a, F, S, SG>>
    for Vec<EventFact>
{
    fn from(value: MacdCrossEvents<'a, F, S, SG>) -> Self {
        let MacdCrossEvents { macds } = value;

        macds
            .as_slice()
            .windows(2)
            .filter_map(|window| {
                let past = &window[0];
                let target = &window[1];
                let pattern = CrossoverPattern::from((
                    past.macd.partial_cmp(&past.signal),
                    target.macd.partial_cmp(&target.signal),
                ));

                match pattern {
                    CrossoverPattern::GoldenCross => Some(EventFact {
                        kind: EventKind::GoldenCross,
                        occurred_at: target.date,
                        event_params: EventParams::Cross {
                            fast_metric: MetricKind::Macd,
                            fast_period: S as u32,
                            slow_metric: MetricKind::Signal,
                            slow_period: SG as u32,
                        },
                    }),
                    CrossoverPattern::DeadCross => Some(EventFact {
                        kind: EventKind::DeadCross,
                        occurred_at: target.date,
                        event_params: EventParams::Cross {
                            fast_metric: MetricKind::Macd,
                            fast_period: S as u32,
                            slow_metric: MetricKind::Signal,
                            slow_period: SG as u32,
                        },
                    }),
                    CrossoverPattern::Neither => None,
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{MACDs, MacdCrossEvents};
    use crate::domain::models::stock::model::Stock;
    use crate::domain::models::technical_analysis::model::{
        DerivedFact, EventFact, EventKind, MetricKind,
    };
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    fn stock(date: NaiveDate, close: f64) -> Stock {
        Stock {
            date,
            close,
            ..Default::default()
        }
    }

    #[test]
    fn converts_macd_and_signal_to_derived_facts() {
        let stocks = (0..30)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0 + i as f64,
                )
            })
            .collect::<Vec<_>>();

        let macds = MACDs::<12, 26, 9>::from(stocks.as_slice());
        let facts: Vec<DerivedFact> = Vec::from(&macds);

        assert!(facts.iter().any(|fact| fact.metric == MetricKind::Macd));
        assert!(facts.iter().any(|fact| fact.metric == MetricKind::Signal));
        assert!(facts.iter().all(|fact| fact.period.is_some()));
        assert!(facts.iter().all(|fact| fact.value.is_finite()));
        assert_eq!(facts.len(), macds.len() * 2);
    }

    #[test]
    fn converts_macd_signal_crossovers_to_events() {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let stocks = (0..60)
            .map(|i| {
                let close = if i < 25 {
                    90.0 - i as f64
                } else {
                    10.0 + (i - 25) as f64 * 2.0
                };
                let date = start_date + chrono::Duration::days(i as i64);
                stock(date, close)
            })
            .collect::<Vec<_>>();

        let macds = MACDs::<12, 26, 9>::from(stocks.as_slice());
        let events: Vec<EventFact> = Vec::from(MacdCrossEvents { macds: &macds });

        assert!(
            events
                .iter()
                .any(|event| event.kind == EventKind::GoldenCross)
        );
    }
}
