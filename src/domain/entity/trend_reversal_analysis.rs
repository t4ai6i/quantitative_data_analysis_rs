use rayon::prelude::*;

use crate::domain::entity::macps::MACPS;
use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::macos::model::Pattern::{Dead, Golden};
use crate::domain::models::macos::model::MACOSES;
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

/// MACOS/MACPS/ECP2/MSESPの売買シグナルから相場転換を分析する
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct TrendReversalAnalysis {
    pub r#type: BuySellSignalType,
    pub macos: Option<(NaiveDate, BuySellSignalType)>,
    pub ecp2: Option<(NaiveDate, BuySellSignalType)>,
    pub msesp: Option<(NaiveDate, BuySellSignalType)>,
    pub macps: Option<(NaiveDate, BuySellSignalType)>,
}

impl TrendReversalAnalysis {
    fn latest_buy_sell_signal_date(
        buy_sell_signal: &[BuySellSignal],
        r#type: &BuySellSignalType,
    ) -> Option<NaiveDate> {
        let filtered: Vec<&BuySellSignal> = buy_sell_signal
            .par_iter()
            .filter(|signal| signal.r#type.eq(r#type))
            .collect();
        filtered
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.date, &b.date))
            .last()
            .map(|signal| signal.date)
    }
}

pub struct TrendReversalAnalysisSet<'a> {
    pub ecp2s: &'a [BuySellSignal],
    pub msesps: &'a [BuySellSignal],
    pub macps: &'a MACPS,
    pub macoses: &'a MACOSES,
}

impl TrendReversalAnalysis {
    fn cmp(
        buy_date: Option<NaiveDate>,
        sell_date: Option<NaiveDate>,
    ) -> Option<(NaiveDate, BuySellSignalType)> {
        match (buy_date, sell_date) {
            (Some(buy), Some(sell)) if buy.ge(&sell) => Some((buy, Buy)),
            (Some(buy), Some(sell)) if sell.ge(&buy) => Some((sell, Sell)),
            (Some(buy), None) => Some((buy, Buy)),
            (None, Some(sell)) => Some((sell, Sell)),
            _ => None,
        }
    }
}

impl<'a> From<&TrendReversalAnalysisSet<'a>> for TrendReversalAnalysis {
    fn from(value: &TrendReversalAnalysisSet<'a>) -> Self {
        let TrendReversalAnalysisSet {
            macps,
            macoses,
            ecp2s,
            msesps,
        } = value;

        let macps = Self::cmp(macps.buy, macps.sell);
        let macos_close_golden = macoses.latest_based_on_close(&Golden);
        let macos_close_dead = macoses.latest_based_on_close(&Dead);
        let macos = Self::cmp(macos_close_golden, macos_close_dead);
        let ecp2_buy = Self::latest_buy_sell_signal_date(ecp2s, &Buy);
        let ecp2_sell = Self::latest_buy_sell_signal_date(ecp2s, &Sell);
        let ecp2 = Self::cmp(ecp2_buy, ecp2_sell);
        let msesp_buy = Self::latest_buy_sell_signal_date(msesps, &Buy);
        let msesp_sell = Self::latest_buy_sell_signal_date(msesps, &Sell);
        let msesp = Self::cmp(msesp_buy, msesp_sell);
        // 終値のGoldenCross/ECP2 Buy/MSESP Buy/MACPS Buy が揃っていれば Buy
        // 終値のDeadCross/ECP2 Sell/MSESP Sell/MACPS Sell が揃っていれば Sell
        // それ以外はStay
        let r#type = match (macos, macps, ecp2, msesp) {
            (Some(macos), Some(macps), Some(ecp2), Some(msesp))
                if macos.1.eq(&Buy) && macps.1.eq(&Buy) && ecp2.1.eq(&Buy) && msesp.1.eq(&Buy) =>
            {
                Buy
            }
            (Some(macos), Some(macps), Some(ecp2), Some(msesp))
                if macos.1.eq(&Sell)
                    && macps.1.eq(&Sell)
                    && ecp2.1.eq(&Sell)
                    && msesp.1.eq(&Sell) =>
            {
                Sell
            }
            _ => Stay,
        };
        Self {
            r#type,
            macps,
            macos,
            ecp2,
            msesp,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::ecp2::VecECP2;
    use crate::domain::entity::macps::MACPS;
    use crate::domain::entity::msesp::VecMSESP;
    use crate::domain::entity::sma::{SMAListPair, SMAListTrio, VecSMA};
    use crate::domain::entity::trend_reversal_analysis::TrendReversalAnalysis;
    use crate::domain::entity::trend_reversal_analysis::TrendReversalAnalysisSet;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::macos::model::MACOSES;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use chrono::NaiveDate;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    const MARUBOZU_MIN_RATE: usize = 90;

    #[test]
    fn trend_reversal_analysis_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let candle_sticks =
            CandleSticks::<MARUBOZU_MIN_RATE>::try_from(vec_stock.as_slice()).unwrap();
        let vec_ecp2 = VecECP2::from(candle_sticks.as_slice());
        let vec_msesp = VecMSESP::from(candle_sticks.as_slice());
        let vec_sma_5 = VecSMA::<5>::from(vec_stock.as_slice());
        let vec_sma_25 = VecSMA::<25>::from(vec_stock.as_slice());
        let vec_sma_50 = VecSMA::<50>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: vec_sma_5.0.as_slice(),
            smas_o: vec_sma_25.0.as_slice(),
        };
        let macoses = MACOSES::from(sma_list_pair);
        let sma_list_trio = SMAListTrio {
            smas_n: vec_sma_5.0.as_slice(),
            smas_o: vec_sma_25.0.as_slice(),
            smas_p: vec_sma_50.0.as_slice(),
        };
        let macps = MACPS::from((vec_stock.as_slice(), sma_list_trio));
        let candle_stick_pattern_set = TrendReversalAnalysisSet {
            ecp2s: vec_ecp2.0.as_slice(),
            msesps: vec_msesp.0.as_slice(),
            macps: &macps,
            macoses: &macoses,
        };
        let actual = TrendReversalAnalysis::from(&candle_stick_pattern_set);
        let expected = TrendReversalAnalysis {
            r#type: Stay,
            macos: NaiveDate::from_ymd_opt(2023, 8, 30).map(|date| (date, Buy)),
            ecp2: NaiveDate::from_ymd_opt(2023, 8, 14).map(|date| (date, Sell)),
            msesp: NaiveDate::from_ymd_opt(2023, 8, 31).map(|date| (date, Buy)),
            macps: NaiveDate::from_ymd_opt(2023, 9, 8).map(|date| (date, Sell)),
        };
        assert_eq!(actual, expected);
    }
}
