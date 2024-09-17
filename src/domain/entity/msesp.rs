use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
use crate::domain::entity::candle_stick::{BullishBearishType, CandleStick};
use itertools::Itertools;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MSESP {
    two_days_before_bullish_bearish: BullishBearishType,
    a_day_before_is_doji: bool,
    target_day_bullish_bearish: BullishBearishType,
}

impl From<MSESP> for BuySellSignalType {
    /// Morning Star/Evening Star Pattern
    ///
    /// ３つのローソク足を考える。
    /// * **Buy**: 対象日の２日前がBearish、対象日の１日前が同事、対象日がBullishのとき
    /// * **Sell**: 対象日の２日前がBullish、対象日の１日前が同事、対象日がBearishのとき
    /// * **Stay**: 上記のどちらにもあてはまならない
    fn from(value: MSESP) -> Self {
        let MSESP {
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

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VecMSESP(pub Vec<BuySellSignal>);

impl<const N: usize> From<&[CandleStick<N>]> for VecMSESP {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::candle_stick::VecCandleStick;
    /// use quantitative_data_analysis_rs::domain::entity::msesp::VecMSESP;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV_9223: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    /// const MARUBOZU_MIN_RATE: usize = 90;
    ///
    /// let vec_stock = Csv::from_slice::<true>(CSV_9223);
    /// let VecCandleStick(vec_candle_stick) = VecCandleStick::<MARUBOZU_MIN_RATE>::from(vec_stock.as_slice());
    /// let _ = VecMSESP::from(vec_candle_stick.as_slice());
    /// ```
    fn from(value: &[CandleStick<N>]) -> Self {
        let vec = value
            .windows(3)
            .map(|candle_sticks| {
                let two_days_before_bullish_bearish =
                    candle_sticks.first().unwrap().bullish_bearish;
                let a_day_before_is_doji = candle_sticks.get(1).unwrap().is_doji;
                let target_day_bullish_bearish = candle_sticks.get(2).unwrap().bullish_bearish;
                let msesp = MSESP {
                    two_days_before_bullish_bearish,
                    a_day_before_is_doji,
                    target_day_bullish_bearish,
                };
                let r#type = BuySellSignalType::from(msesp);
                let today = candle_sticks.get(2).unwrap().date;
                BuySellSignal {
                    r#type,
                    date: today,
                }
            })
            .collect_vec();
        VecMSESP(vec)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::buy_sell_signal::BuySellSignalType::Sell;
    use crate::domain::entity::buy_sell_signal::{
        BuySellSignal,
        BuySellSignalType::{Buy, Stay},
    };
    use crate::domain::entity::candle_stick::VecCandleStick;
    use crate::domain::entity::msesp::VecMSESP;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use chrono::NaiveDate;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const MARUBOZU_MIN_RATE: usize = 90;

    #[test]
    fn from_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let VecCandleStick(vec_candle_stick) =
            VecCandleStick::<MARUBOZU_MIN_RATE>::from(vec_stock.as_slice());
        let VecMSESP(vec_msesp) = VecMSESP::from(vec_candle_stick.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) = vec_msesp
            .into_iter()
            .filter(|signal| !signal.r#type.eq(&Stay))
            .partition(|signal| signal.r#type.eq(&Buy));
        let (expected_buy, expected_sell): (Vec<BuySellSignal>, Vec<BuySellSignal>) = (
            vec![
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 9, 28).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 9, 29).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2022, 10, 13).unwrap(),
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
                    date: NaiveDate::from_ymd_opt(2022, 12, 26).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 1, 5).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 1, 24).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 2, 15).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 3, 1).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 3, 7).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 3, 17).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 3, 29).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 6, 9).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 6, 14).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 8, 22).unwrap(),
                },
                BuySellSignal {
                    r#type: Buy,
                    date: NaiveDate::from_ymd_opt(2023, 8, 31).unwrap(),
                },
            ],
            vec![
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 10, 11).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 11, 8).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 11, 9).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2022, 11, 24).unwrap(),
                },
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
                    date: NaiveDate::from_ymd_opt(2023, 3, 20).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 3, 31).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 4, 21).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 5, 24).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 5, 31).unwrap(),
                },
                BuySellSignal {
                    r#type: Sell,
                    date: NaiveDate::from_ymd_opt(2023, 6, 30).unwrap(),
                },
            ],
        );
        assert_eq!(actual_buy, expected_buy);
        assert_eq!(actual_sell, expected_sell);
    }
}
