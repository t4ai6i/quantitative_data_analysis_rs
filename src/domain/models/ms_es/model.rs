use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::candle_stick::model::{BullishBearishType, CandleStick};
use crate::domain::models::technical_analysis::model::{EventFact, EventKind, EventParams};
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MsEs {
    two_days_before_bullish_bearish: BullishBearishType,
    a_day_before_is_doji: bool,
    target_day_bullish_bearish: BullishBearishType,
}

impl From<MsEs> for BuySellSignalType {
    /// Morning Star/Evening Star
    ///
    /// ３つのローソク足を考える。
    /// * **Buy**: 対象日の２日前がBearish、対象日の１日前が同事、対象日がBullishのとき
    /// * **Sell**: 対象日の２日前がBullish、対象日の１日前が同事、対象日がBearishのとき
    /// * **Stay**: 上記のどちらにもあてはまならない
    fn from(value: MsEs) -> Self {
        let MsEs {
            two_days_before_bullish_bearish,
            a_day_before_is_doji,
            target_day_bullish_bearish,
        } = value;
        match (
            two_days_before_bullish_bearish,
            a_day_before_is_doji,
            target_day_bullish_bearish,
        ) {
            (BullishBearishType::Bearish, true, BullishBearishType::Bullish) => Self::Buy,
            (BullishBearishType::Bullish, true, BullishBearishType::Bearish) => Self::Sell,
            _ => Self::Stay,
        }
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deref, DerefMut)]
pub struct MsEses(Vec<BuySellSignal>);

impl From<&[CandleStick]> for MsEses {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    ///
    /// use quantitative_data_analysis_rs::domain::models::candle_stick::model::CandleSticks;
    /// use quantitative_data_analysis_rs::domain::models::stock::model;
    /// use quantitative_data_analysis_rs::domain::models::ms_es::model::MsEses;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
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
    ///   let mseses = MsEses::from(candle_sticks.as_slice());
    ///   assert_eq!(mseses.len(), 33);
    /// });
    /// ```
    fn from(value: &[CandleStick]) -> Self {
        let vec_ms_es = value
            .par_windows(3)
            .map(|candle_sticks| {
                let two_days_before_bullish_bearish = candle_sticks[0].bullish_bearish;
                let a_day_before_is_doji = candle_sticks[1].is_doji;
                let target_day_bullish_bearish = candle_sticks[2].bullish_bearish;
                let ms_es = MsEs {
                    two_days_before_bullish_bearish,
                    a_day_before_is_doji,
                    target_day_bullish_bearish,
                };
                let r#type = BuySellSignalType::from(ms_es);
                let today = candle_sticks[2].date;
                BuySellSignal {
                    r#type,
                    date: today,
                }
            })
            .collect();
        MsEses(vec_ms_es)
    }
}

pub struct MsEsEvents<'a> {
    pub mseses: &'a MsEses,
}

impl From<MsEsEvents<'_>> for Vec<EventFact> {
    fn from(value: MsEsEvents<'_>) -> Self {
        let MsEsEvents { mseses } = value;

        mseses
            .iter()
            .filter_map(|signal| match signal.r#type {
                BuySellSignalType::Buy => Some(EventFact {
                    kind: EventKind::MorningStar,
                    occurred_at: signal.date,
                    event_params: EventParams::Pattern { window_bars: 3 },
                }),
                BuySellSignalType::Sell => Some(EventFact {
                    kind: EventKind::EveningStar,
                    occurred_at: signal.date,
                    event_params: EventParams::Pattern { window_bars: 3 },
                }),
                BuySellSignalType::Stay => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType::Sell;
    use crate::domain::models::buy_sell_signal::model::{
        BuySellSignal,
        BuySellSignalType::{Buy, Stay},
    };
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::ms_es::model::{MsEsEvents, MsEses};
    use crate::domain::models::technical_analysis::model::{EventFact, EventKind};
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;
    use anyhow::Result;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use rayon::prelude::*;

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
        let MsEses(vec_ms_es) = MsEses::from(candle_sticks.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) = vec_ms_es
            .into_par_iter()
            .filter(|signal| signal.r#type.ne(&Stay))
            .partition(|signal| signal.r#type.eq(&Buy));
        let (expected_buy, expected_sell): (Vec<BuySellSignal>, Vec<BuySellSignal>) = (
            vec![
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 9, 29).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 10, 14).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 10, 26).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 10, 31).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 11, 30).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 1, 24).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 3, 7).unwrap(),
                },
            ],
            vec![
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 12, 14).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 2, 22).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 4, 21).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 5, 24).unwrap(),
                },
            ],
        );
        assert_eq!(actual_buy, expected_buy);
        assert_eq!(actual_sell, expected_sell);
        Ok(())
    }

    #[test]
    fn converts_morning_and_evening_stars_to_events() {
        let mses = MsEses(vec![
            BuySellSignal {
                r#type: Buy,
                date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
            },
            BuySellSignal {
                r#type: Sell,
                date: NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(),
            },
        ]);

        let events: Vec<EventFact> = Vec::from(MsEsEvents { mseses: &mses });

        assert!(
            events
                .iter()
                .any(|event| event.kind == EventKind::MorningStar)
        );
        assert!(
            events
                .iter()
                .any(|event| event.kind == EventKind::EveningStar)
        );
    }
}
