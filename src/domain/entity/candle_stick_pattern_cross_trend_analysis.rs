use std::ops::Sub;

use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
use crate::domain::entity::candle_stick_patterns_crosses_set::CandleStickPatternsCrossesSet;
use crate::domain::entity::cross::{Cross, CrossDirectionType};

/// Signal発生日と最も近いCross発生日のペア
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CrossDateNearestSignalDate {
    pub cross_date: NaiveDate,
    pub signal_date: NaiveDate,
}

/// ローソク足パターンとクロス方向を基にしたトレンド解析
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CandleStickPatternCrossTrendAnalysis {
    pub ecp2_buy_golden: Option<CrossDateNearestSignalDate>,
    pub ecp2_sell_dead: Option<CrossDateNearestSignalDate>,
}

impl CandleStickPatternCrossTrendAnalysis {
    /// Signal発生日と最も近いCross発生日のペアを取得
    fn get_signal_cross_direction_pair(
        crosses: &[Cross],
        signals: &[BuySellSignal],
        cross_direction_type: CrossDirectionType,
        buy_sell_signal_type: BuySellSignalType,
    ) -> Option<CrossDateNearestSignalDate> {
        signals
            .iter()
            .filter(|signal| signal.r#type.eq(&buy_sell_signal_type))
            // 最新のSignalを取得
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .collect_vec()
            .last()
            .and_then(|signal| {
                crosses
                    .iter()
                    // Signal発生日から最も近いCross発生日を取得
                    .filter_map(|cross| {
                        if cross
                            .cross_direction_5_25
                            .close_average
                            .ne(&cross_direction_type)
                        {
                            return None;
                        }
                        Some((cross.date, cross.date.sub(signal.date).abs()))
                    })
                    .sorted_by(|a, b| {
                        let a = a.1;
                        let b = b.1;
                        Ord::cmp(&a, &b)
                    })
                    .collect_vec()
                    .first()
                    .map(|t| CrossDateNearestSignalDate {
                        cross_date: t.0,
                        signal_date: signal.date,
                    })
            })
    }
}

impl<'a> From<&CandleStickPatternsCrossesSet<'a>> for CandleStickPatternCrossTrendAnalysis {
    fn from(value: &CandleStickPatternsCrossesSet<'a>) -> Self {
        let CandleStickPatternsCrossesSet { ecp2s, crosses } = value;

        // ecp2の最新のBuySignal発生日に最も近いGoldenCross発生日を取得する
        let ecp2_buy = Self::get_signal_cross_direction_pair(
            crosses,
            ecp2s,
            CrossDirectionType::Golden,
            BuySellSignalType::Buy,
        );

        // ecp2の最新のSellSignal発生日に最も近いDeadCross発生日を取得する
        let ecp2_sell = Self::get_signal_cross_direction_pair(
            crosses,
            ecp2s,
            CrossDirectionType::Dead,
            BuySellSignalType::Sell,
        );

        // TODO: 同じように最新のMorningStar発生日に最も近いGoldenCross発生日を取得する
        // TODO: 同じように最新のEveningStar発生日に最も近いDeadCross発生日を取得する
        Self {
            ecp2_buy_golden: ecp2_buy,
            ecp2_sell_dead: ecp2_sell,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::domain::entity::candle_stick::VecCandleStick;
    use crate::domain::entity::candle_stick_pattern_cross_trend_analysis::{
        CandleStickPatternCrossTrendAnalysis, CrossDateNearestSignalDate,
    };
    use crate::domain::entity::candle_stick_patterns_crosses_set::CandleStickPatternsCrossesSet;
    use crate::domain::entity::cross::VecCross;
    use crate::domain::entity::ecp2::VecECP2;
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const MARUBOZU_MIN_RATE: usize = 90;

    #[test]
    fn candle_stick_pattern_cross_trend_analysis_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
        let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(sma_list_pair);
        let VecCandleStick(candle_sticks) =
            VecCandleStick::<MARUBOZU_MIN_RATE>::from(vec_stock.as_slice());

        let VecECP2(buy_sell_signals) = VecECP2::from(candle_sticks.as_slice());
        let candle_stick_patterns_crosses_set = CandleStickPatternsCrossesSet {
            ecp2s: buy_sell_signals.as_slice(),
            crosses: crosses.as_slice(),
        };
        let actual = CandleStickPatternCrossTrendAnalysis::from(&candle_stick_patterns_crosses_set);
        let expected = CandleStickPatternCrossTrendAnalysis {
            ecp2_buy_golden: Some(CrossDateNearestSignalDate {
                signal_date: NaiveDate::from_ymd_opt(2023, 6, 6).unwrap(),
                cross_date: NaiveDate::from_ymd_opt(2023, 6, 15).unwrap(),
            }),
            ecp2_sell_dead: Some(CrossDateNearestSignalDate {
                signal_date: NaiveDate::from_ymd_opt(2023, 8, 14).unwrap(),
                cross_date: NaiveDate::from_ymd_opt(2023, 8, 17).unwrap(),
            }),
        };
        assert_eq!(actual, expected);
    }
}
