use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::candle_stick::model::{BullishBearishType, CandleStick};
use crate::domain::models::technical_analysis::model::{
    DirectionType, EventFact, EventKind, EventParams,
};
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
use std::cmp::Ordering;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BodyEngulfing {
    body_low_ordering: Option<Ordering>,
    body_high_ordering: Option<Ordering>,
    prev_bullish_bearish: BullishBearishType,
    today_bullish_bearish: BullishBearishType,
}

/// Body Engulfing
///
/// ある日の始値と終値の間にその前日の始値と終値が飲み込まれているかをみる
///
/// * **Buy**: ある日の始値と終値の間にその前日の始値と終値が飲み込まれており、前日がBearishCandleで当日がBullishCandleのとき
/// * **Sell**: ある日の始値と終値の間にその前日の始値と終値が飲み込まれており、前日がBullishCandleで当日がBearishCandleのとき
/// * **Stay**: 上記のどちらにもあてはまならない
impl From<BodyEngulfing> for BuySellSignalType {
    fn from(value: BodyEngulfing) -> Self {
        let BodyEngulfing {
            body_low_ordering,
            body_high_ordering,
            prev_bullish_bearish,
            today_bullish_bearish,
        } = value;
        match (
            body_low_ordering,
            body_high_ordering,
            prev_bullish_bearish,
            today_bullish_bearish,
        ) {
            (
                Some(Ordering::Less),
                Some(Ordering::Greater),
                BullishBearishType::Bearish,
                BullishBearishType::Bullish,
            ) => Self::Buy,
            (
                Some(Ordering::Less),
                Some(Ordering::Greater),
                BullishBearishType::Bullish,
                BullishBearishType::Bearish,
            ) => Self::Sell,
            _ => Self::Stay,
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deref, DerefMut)]
pub struct BodyEngulfings(Vec<BuySellSignal>);

impl From<&[CandleStick]> for BodyEngulfings {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    ///
    /// use quantitative_data_analysis_rs::domain::models::candle_stick::model::CandleSticks;
    /// use quantitative_data_analysis_rs::domain::models::body_engulfing::model::BodyEngulfings;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    ///
    /// const MARUBOZU_BODY_MIN_RATIO: usize = 90;
    /// const MARUBOZU_WICK_MAX_RATIO: usize = 2;
    /// const DOJI_MAX_BODY_RATIO: usize = 5;
    /// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(&query).await.unwrap();
    ///   let candle_sticks = CandleSticks::<MARUBOZU_BODY_MIN_RATIO, MARUBOZU_WICK_MAX_RATIO, DOJI_MAX_BODY_RATIO>::try_from(stocks.as_slice()).unwrap();
    ///   let body_engulfings = BodyEngulfings::from(candle_sticks.as_slice());
    ///   assert_eq!(body_engulfings.len(), 34);
    /// });
    /// ```
    fn from(value: &[CandleStick]) -> Self {
        let vec = value
            .par_windows(2)
            .map(|candle_sticks| {
                let prev = candle_sticks.first().unwrap();
                let today = candle_sticks.get(1).unwrap();

                let body_high_ordering = today.body_high.partial_cmp(&prev.body_high);
                let body_low_ordering = today.body_low.partial_cmp(&prev.body_low);
                let body_engulfing = BodyEngulfing {
                    body_low_ordering,
                    body_high_ordering,
                    prev_bullish_bearish: prev.bullish_bearish,
                    today_bullish_bearish: today.bullish_bearish,
                };
                let r#type = BuySellSignalType::from(body_engulfing);
                let date = today.date;
                BuySellSignal { r#type, date }
            })
            .collect();
        BodyEngulfings(vec)
    }
}

pub struct BodyEngulfingEvents<'a> {
    pub body_engulfings: &'a BodyEngulfings,
}

impl From<BodyEngulfingEvents<'_>> for Vec<EventFact> {
    fn from(value: BodyEngulfingEvents<'_>) -> Self {
        let BodyEngulfingEvents { body_engulfings } = value;

        body_engulfings
            .iter()
            .filter_map(|signal| match signal.r#type {
                BuySellSignalType::Buy => Some(EventFact {
                    kind: EventKind::BodyEngulfing,
                    occurred_at: signal.date,
                    direction: DirectionType::Uptrend,
                    event_params: EventParams::Pattern {
                        pattern_name: EventKind::BodyEngulfing,
                        window_bars: 2,
                    },
                }),
                BuySellSignalType::Sell => Some(EventFact {
                    kind: EventKind::BodyEngulfing,
                    occurred_at: signal.date,
                    direction: DirectionType::Downtrend,
                    event_params: EventParams::Pattern {
                        pattern_name: EventKind::BodyEngulfing,
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

    use crate::domain::models::body_engulfing::model::{BodyEngulfingEvents, BodyEngulfings};
    use crate::domain::models::buy_sell_signal::model::tests::TupleVecBuySellSignal;
    use crate::domain::models::buy_sell_signal::model::{
        BuySellSignal,
        BuySellSignalType::{Buy, Sell},
    };
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::technical_analysis::model::{EventFact, EventKind};
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const MARUBOZU_BODY_MIN_RATIO: usize = 90;
    const MARUBOZU_WICK_MAX_RATIO: usize = 2;
    const DOJI_MAX_BODY_RATIO: usize = 5;
    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn from_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(&query).await?;
        let candle_sticks = CandleSticks::<
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
        >::try_from(stocks.as_slice())?;
        let body_engulfings = BodyEngulfings::from(candle_sticks.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) =
            TupleVecBuySellSignal::from(body_engulfings.0.as_slice()).0;
        let (expected_buy, expected_sell): (Vec<BuySellSignal>, Vec<BuySellSignal>) = (
            vec![
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 12, 9).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 1, 10).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 2, 7).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 3, 23).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 4, 7).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 6, 6).unwrap(),
                },
            ],
            vec![
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 10, 11).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 11, 1).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 11, 28).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 1, 30).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 3, 2).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 6, 23).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 7, 10).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 8, 14).unwrap(),
                },
            ],
        );
        assert_eq!(actual_buy, expected_buy);
        assert_eq!(actual_sell, expected_sell);
        Ok(())
    }

    #[test]
    fn converts_body_engulfing_signals_to_events() {
        let body_engulfings = BodyEngulfings(vec![
            BuySellSignal {
                r#type: Buy,
                date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            },
            BuySellSignal {
                r#type: Sell,
                date: NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(),
            },
        ]);

        let events: Vec<EventFact> = Vec::from(BodyEngulfingEvents {
            body_engulfings: &body_engulfings,
        });

        assert!(
            events
                .iter()
                .any(|event| event.kind == EventKind::BodyEngulfing)
        );
    }
}
