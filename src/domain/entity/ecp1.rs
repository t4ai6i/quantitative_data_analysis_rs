use std::cmp::Ordering;

use itertools::Itertools;

use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
use crate::domain::entity::stock::Stock;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ECP1 {
    low_ordering: Option<Ordering>,
    high_ordering: Option<Ordering>,
}

/// Engulfing Candlestick Pattern1
///
/// ある日とその前日の安値・高値の切り上がり・切り下がりをみる
///
/// * **Buy**: 安値切り上がりかつ高値切り上がり
/// * **Sell**: 安値切り下がりかつ高値切り下がり
/// * **Stay**: 上記のどちらにもあてはまならない
impl From<ECP1> for BuySellSignalType {
    fn from(value: ECP1) -> Self {
        let ECP1 {
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

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct VecECP1(pub Vec<BuySellSignal>);

impl From<&[Stock]> for VecECP1 {
    ///
    /// # Examples
    /// ```
    /// use quantitative_data_analysis_rs::domain::entity::ecp1::VecECP1;
    /// use quantitative_data_analysis_rs::infrastructure::from_slice::FromSlice;
    /// use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
    ///
    /// const CSV_9223: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    ///
    /// let vec_stock = Csv::from_slice::<true>(CSV_9223);
    /// let _ = VecECP1::from(vec_stock.as_slice());
    /// ```
    fn from(value: &[Stock]) -> Self {
        let vec = value
            .windows(2)
            .map(|stock| {
                let prev = stock.first().unwrap();
                let today = stock.get(1).unwrap();
                // 前日・当日それぞれの安値を比較する
                let low_ordering = today.low.partial_cmp(&prev.low);
                // 前日・当日それぞれの高値を比較する
                let high_ordering = today.high.partial_cmp(&prev.high);
                let ecp1 = ECP1 {
                    low_ordering,
                    high_ordering,
                };
                // 高値・安値の切り上げ・切り下げを基にした売買シグナル
                let r#type = BuySellSignalType::from(ecp1);
                let date = today.date;
                BuySellSignal { r#type, date }
            })
            .collect_vec();
        VecECP1(vec)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
    use crate::domain::entity::ecp1::VecECP1;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;

    const CSV_9223: &[u8] = include_bytes!("../../../assets/9223.T.csv");
    #[test]
    fn from_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_9223);
        let vec_ecp1 = VecECP1::from(vec_stock.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) = vec_ecp1
            .0
            .into_iter()
            .filter(|signal| !signal.r#type.eq(&BuySellSignalType::Stay))
            .partition(|signal| signal.r#type.eq(&BuySellSignalType::Buy));
        let expected: (Vec<BuySellSignal>, Vec<BuySellSignal>) = (
            vec![
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2023, 12, 28).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2023, 12, 29).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2024, 1, 19).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2024, 2, 2).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2024, 2, 5).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Buy,
                    date: NaiveDate::from_ymd_opt(2024, 2, 14).unwrap(),
                },
            ],
            vec![
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2023, 12, 27).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 11).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 18).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 23).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 24).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 25).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 1, 26).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 7).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 8).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 15).unwrap(),
                },
                BuySellSignal {
                    r#type: BuySellSignalType::Sell,
                    date: NaiveDate::from_ymd_opt(2024, 2, 16).unwrap(),
                },
            ],
        );
        assert_eq!(actual_buy, expected.0);
        assert_eq!(actual_sell, expected.1);
    }
}
