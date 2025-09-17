use deref_derive::{Deref, DerefMut};
use itertools::Itertools;
use std::cmp::Ordering;

use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::candle_stick::model::{BullishBearishType, CandleStick};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ECP2 {
    body_low_ordering: Option<Ordering>,
    body_high_ordering: Option<Ordering>,
    prev_bullish_bearish: BullishBearishType,
    today_bullish_bearish: BullishBearishType,
}

/// Engulfing Candlestick Pattern2
///
/// ある日の始値と終値の間にその前日の始値と終値が飲み込まれているかをみる
///
/// * **Buy**: ある日の始値と終値の間にその前日の始値と終値が飲み込まれており、前日がBearishCandleで当日がBullishCandleのとき
/// * **Sell**: ある日の始値と終値の間にその前日の始値と終値が飲み込まれており、前日がBullishCandleで当日がBearishCandleのとき
/// * **Stay**: 上記のどちらにもあてはまならない
impl From<ECP2> for BuySellSignalType {
    fn from(value: ECP2) -> Self {
        let ECP2 {
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
pub struct ECP2s(Vec<BuySellSignal>);

impl<const N: usize> From<&[CandleStick<N>]> for ECP2s {
    ///
    /// # Examples
    /// ```
    /// use bytes::Bytes;
    ///
    /// use quantitative_data_analysis_rs::domain::models::candle_stick::model::CandleSticks;
    /// use quantitative_data_analysis_rs::domain::models::ecp2::model::ECP2s;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::queries;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    ///
    /// const MARUBOZU_MIN_RATE: usize = 90;
    /// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
    ///   let query = queries::get_stocks::Query {
    ///     ..Default::default()
    ///   };
    ///   let stocks = dsv.get_stocks(query).await.unwrap();
    ///   let candle_sticks = CandleSticks::<MARUBOZU_MIN_RATE>::try_from(stocks.as_slice()).unwrap();
    ///   let ecp2s = ECP2s::from(candle_sticks.as_slice());
    ///   assert_eq!(ecp2s.len(), 34);
    /// });
    /// ```
    fn from(value: &[CandleStick<N>]) -> Self {
        let vec = value
            .windows(2)
            .map(|candle_sticks| {
                let prev = candle_sticks.first().unwrap();
                let today = candle_sticks.get(1).unwrap();

                let body_high_ordering = today.body_high.partial_cmp(&prev.body_high);
                let body_low_ordering = today.body_low.partial_cmp(&prev.body_low);
                let ecp2 = ECP2 {
                    body_low_ordering,
                    body_high_ordering,
                    prev_bullish_bearish: prev.bullish_bearish,
                    today_bullish_bearish: today.bullish_bearish,
                };
                let r#type = BuySellSignalType::from(ecp2);
                let date = today.date;
                BuySellSignal { r#type, date }
            })
            .collect_vec();
        ECP2s(vec)
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
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::ecp2::model::ECP2s;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const MARUBOZU_MIN_RATE: usize = 90;
    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn from_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(query).await?;
        let candle_sticks = CandleSticks::<MARUBOZU_MIN_RATE>::try_from(stocks.as_slice())?;
        let vec_ecp2 = ECP2s::from(candle_sticks.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) =
            TupleVecBuySellSignal::from(vec_ecp2.0.as_slice()).0;
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
}
