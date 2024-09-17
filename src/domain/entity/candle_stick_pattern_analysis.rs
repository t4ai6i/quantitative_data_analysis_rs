use crate::domain::entity::buy_sell_signal::BuySellSignalType::{Buy, Sell};
use crate::domain::entity::buy_sell_signal::{BuySellSignal, BuySellSignalType};
use crate::domain::entity::candle_stick_pattern_set::CandleStickPatternSet;
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// ローソク足を基にしたテクニカル解析
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CandleStickPatternAnalysis {
    pub latest_ecp2_buy: Option<NaiveDate>,
    pub latest_ecp2_sell: Option<NaiveDate>,
    pub latest_msesp_buy: Option<NaiveDate>,
    pub latest_msesp_sell: Option<NaiveDate>,
}

impl CandleStickPatternAnalysis {
    fn latest_buy_sell_signal_date(
        buy_sell_signal: &[BuySellSignal],
        r#type: &BuySellSignalType,
    ) -> Option<NaiveDate> {
        buy_sell_signal
            .iter()
            .filter(|signal| signal.r#type.eq(r#type))
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .rev()
            .last()
            .map(|signal| signal.date)
    }
}

impl<'a> From<&CandleStickPatternSet<'a>> for CandleStickPatternAnalysis {
    fn from(value: &CandleStickPatternSet<'a>) -> Self {
        let CandleStickPatternSet { ecp2s, msesps } = value;

        // ECP2の最新のBuySignal発生日を取得する
        let latest_ecp2_buy = Self::latest_buy_sell_signal_date(ecp2s, &Buy);
        // ECP2の最新のSellSignal発生日を取得する
        let latest_ecp2_sell = Self::latest_buy_sell_signal_date(ecp2s, &Sell);
        // MSESPの最新のBuySignal発生日を取得する
        let latest_msesp_buy = Self::latest_buy_sell_signal_date(msesps, &Buy);
        // MSESPの最新のSellSignal発生日を取得する
        let latest_msesp_sell = Self::latest_buy_sell_signal_date(msesps, &Sell);
        Self {
            latest_ecp2_buy,
            latest_ecp2_sell,
            latest_msesp_buy,
            latest_msesp_sell,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::candle_stick::VecCandleStick;
    use crate::domain::entity::candle_stick_pattern_analysis::CandleStickPatternAnalysis;
    use crate::domain::entity::candle_stick_pattern_set::CandleStickPatternSet;
    use crate::domain::entity::ecp2::VecECP2;
    use crate::domain::entity::msesp::VecMSESP;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use chrono::NaiveDate;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const MARUBOZU_MIN_RATE: usize = 90;

    #[test]
    fn candle_stick_pattern_analysis_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let VecCandleStick(candle_sticks) =
            VecCandleStick::<MARUBOZU_MIN_RATE>::from(vec_stock.as_slice());
        let vec_ecp2 = VecECP2::from(candle_sticks.as_slice());
        let vec_msesp = VecMSESP::from(candle_sticks.as_slice());
        let candle_stick_pattern_set = CandleStickPatternSet {
            ecp2s: vec_ecp2.0.as_slice(),
            msesps: vec_msesp.0.as_slice(),
        };
        let actual = CandleStickPatternAnalysis::from(&candle_stick_pattern_set);
        let expected = CandleStickPatternAnalysis {
            latest_ecp2_buy: NaiveDate::from_ymd_opt(2022, 12, 9),
            latest_ecp2_sell: NaiveDate::from_ymd_opt(2022, 10, 11),
            latest_msesp_buy: NaiveDate::from_ymd_opt(2022, 9, 28),
            latest_msesp_sell: NaiveDate::from_ymd_opt(2022, 10, 11),
        };
        assert_eq!(actual, expected);
    }
}
