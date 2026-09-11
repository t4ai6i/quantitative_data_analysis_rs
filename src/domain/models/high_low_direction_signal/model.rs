use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{
    DirectionType, EventFact, EventKind, EventParams,
};
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use std::cmp::Ordering;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct HighLowDirectionSignal {
    low_ordering: Option<Ordering>,
    high_ordering: Option<Ordering>,
}

/// High Low Direction Signal
///
/// ある日とその前日の安値・高値の切り上がり・切り下がりをみて、価格レンジのトレンド方向のシグナルを出す
///
/// * **Buy**: 安値切り上がりかつ高値切り上がり
/// * **Sell**: 安値切り下がりかつ高値切り下がり
/// * **Stay**: 上記のどちらにもあてはまならない
impl From<HighLowDirectionSignal> for BuySellSignalType {
    fn from(value: HighLowDirectionSignal) -> Self {
        let HighLowDirectionSignal {
            low_ordering,
            high_ordering,
        } = value;
        match (low_ordering, high_ordering) {
            (Some(Ordering::Greater), Some(Ordering::Greater)) => Self::Buy,
            (Some(Ordering::Less), Some(Ordering::Less)) => Self::Sell,
            _ => Self::Stay,
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deref, DerefMut)]
pub struct HighLowDirectionSignals(Vec<BuySellSignal>);

impl From<&[Stock]> for HighLowDirectionSignals {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    ///
    /// use quantitative_data_analysis_rs::domain::models::high_low_direction_signal::model::HighLowDirectionSignals;
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let high_low_direction_signals = HighLowDirectionSignals::from(stocks.as_slice());
    ///   assert_eq!(high_low_direction_signals.len(), 34);
    /// });
    /// ```
    fn from(value: &[Stock]) -> Self {
        let vec_high_low_direction_signal = value
            .par_windows(2)
            .map(|stock| {
                let prev = stock.first().unwrap();
                let today = stock.get(1).unwrap();
                // 前日・当日それぞれの安値を比較する
                let low_ordering = today.low.partial_cmp(&prev.low);
                // 前日・当日それぞれの高値を比較する
                let high_ordering = today.high.partial_cmp(&prev.high);
                let high_low_direction_signal = HighLowDirectionSignal {
                    low_ordering,
                    high_ordering,
                };
                // 高値・安値の切り上げ・切り下げを基にした売買シグナル
                let r#type = BuySellSignalType::from(high_low_direction_signal);
                let date = today.date;
                BuySellSignal { r#type, date }
            })
            .collect();
        HighLowDirectionSignals(vec_high_low_direction_signal)
    }
}

pub struct HighLowDirectionEvents<'a> {
    pub signals: &'a HighLowDirectionSignals,
}

impl From<HighLowDirectionEvents<'_>> for Vec<EventFact> {
    fn from(value: HighLowDirectionEvents<'_>) -> Self {
        let HighLowDirectionEvents { signals } = value;

        signals
            .iter()
            .filter_map(|signal| match signal.r#type {
                BuySellSignalType::Buy => Some(EventFact {
                    kind: EventKind::HighLowDirection,
                    occurred_at: signal.date,
                    direction: DirectionType::Uptrend,
                    event_params: EventParams::Pattern {
                        pattern_name: EventKind::HighLowDirection,
                        window_bars: 2,
                    },
                }),
                BuySellSignalType::Sell => Some(EventFact {
                    kind: EventKind::HighLowDirection,
                    occurred_at: signal.date,
                    direction: DirectionType::Downtrend,
                    event_params: EventParams::Pattern {
                        pattern_name: EventKind::HighLowDirection,
                        window_bars: 2,
                    },
                }),
                BuySellSignalType::Stay => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::domain::models::buy_sell_signal::model::tests::TupleVecBuySellSignal;
    use crate::domain::models::buy_sell_signal::model::{
        BuySellSignal,
        BuySellSignalType::{Buy, Sell},
    };
    use crate::domain::models::high_low_direction_signal::model::{
        HighLowDirectionEvents, HighLowDirectionSignals,
    };
    use crate::domain::models::technical_analysis::model::{EventFact, EventKind};
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");

    #[tokio::test]
    async fn from_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(&query).await?;
        let high_low_direction_signals = HighLowDirectionSignals::from(stocks.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) =
            TupleVecBuySellSignal::from(high_low_direction_signals.as_slice()).0;
        let (expected_buy, expected_sell): (Vec<BuySellSignal>, Vec<BuySellSignal>) = (
            vec![
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 12, 28).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 12, 29).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2024, 1, 19).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2024, 2, 2).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2024, 2, 5).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
                },
            ],
            vec![
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 12, 27).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 23).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 24).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 26).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 7).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 8).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
                },
            ],
        );
        assert_eq!(actual_buy, expected_buy);
        assert_eq!(actual_sell, expected_sell);
        Ok(())
    }

    #[test]
    fn converts_high_low_direction_signals_to_events() {
        let signals = HighLowDirectionSignals(vec![
            BuySellSignal {
                r#type: Buy,
                date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            },
            BuySellSignal {
                r#type: Sell,
                date: NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(),
            },
        ]);

        let events: Vec<EventFact> = Vec::from(HighLowDirectionEvents { signals: &signals });

        assert!(
            events
                .iter()
                .any(|event| event.kind == EventKind::HighLowDirection)
        );
    }
}
