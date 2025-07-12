use deref_derive::{Deref, DerefMut};
use itertools::Itertools;
use std::cmp::Ordering;

use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::stock::model::Stock;

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

#[derive(Default, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Deref, DerefMut)]
pub struct ECP1s(Vec<BuySellSignal>);

impl From<&[Stock]> for ECP1s {
    ///
    /// # Examples
    /// ```
    /// use chrono::NaiveDate;
    ///
    /// use quantitative_data_analysis_rs::domain::models::ecp1::model::ECP1s;
    /// use quantitative_data_analysis_rs::domain::repositories::stock::repository::Stock;
    /// use quantitative_data_analysis_rs::infrastructure::dsv::Dsv;
    /// use quantitative_data_analysis_rs::infrastructure::repositories::stock::structures::internal::csv;
    ///
    /// const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");
    ///
    /// tokio_test::block_on(async {
    ///   let dsv = Dsv::<csv::Structure>::new(true, CSV.to_vec());
    ///   let default_str = "";
    ///   let stocks = dsv.get_stocks(default_str, default_str, NaiveDate::default (), NaiveDate::default ()).await.unwrap();
    ///   let ecp1s = ECP1s::from(stocks.as_slice());
    ///   assert_eq!(ecp1s.len(), 34);
    /// });
    /// ```
    fn from(value: &[Stock]) -> Self {
        let vec_buy_sell_signal = value
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
        ECP1s(vec_buy_sell_signal)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::domain::models::buy_sell_signal::model::tests::TupleVecBuySellSignal;
    use crate::domain::models::buy_sell_signal::model::{
        BuySellSignal,
        BuySellSignalType::{Buy, Sell},
    };
    use crate::domain::models::ecp1::model::ECP1s;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const CSV: &[u8] = include_bytes!("../../../../assets/9223.T.csv");

    #[tokio::test]
    async fn from_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, CSV.to_vec());
        let default_str = "";
        let stocks = dsv
            .get_stocks(
                default_str,
                default_str,
                NaiveDate::default(),
                NaiveDate::default(),
            )
            .await?;
        let ecp1s = ECP1s::from(stocks.as_slice());
        let (actual_buy, actual_sell): (Vec<_>, Vec<_>) =
            TupleVecBuySellSignal::from(ecp1s.as_slice()).0;
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
}
