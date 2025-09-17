use rayon::prelude::*;

use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::macos::model::MACOSes;
use crate::domain::models::macos::model::Pattern::{DeadCross, GoldenCross};
use crate::domain::models::macps::model::MACPS;
use chrono::NaiveDate;
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
        buy_sell_signal
            .par_iter()
            .filter(|signal| signal.r#type.eq(r#type))
            .max_by(|a, b| Ord::cmp(&a.date, &b.date))
            .map(|signal| signal.date)
    }

    fn resolve_latest_signal(
        buy_date: Option<NaiveDate>,
        sell_date: Option<NaiveDate>,
    ) -> Option<(NaiveDate, BuySellSignalType)> {
        match (buy_date, sell_date) {
            (Some(buy_date), Some(sell_date)) => {
                if buy_date >= sell_date {
                    Some((buy_date, Buy))
                } else {
                    Some((sell_date, Sell))
                }
            }
            (Some(buy_date), None) => Some((buy_date, Buy)),
            (None, Some(sell_date)) => Some((sell_date, Sell)),
            _ => None,
        }
    }
}

pub struct TrendReversalAnalysisSet<'a> {
    pub ecp2s: &'a [BuySellSignal],
    pub msesps: &'a [BuySellSignal],
    pub macps: &'a MACPS,
    pub macoses: &'a MACOSes,
}

impl<'a> From<TrendReversalAnalysisSet<'a>> for TrendReversalAnalysis {
    fn from(value: TrendReversalAnalysisSet<'a>) -> Self {
        let TrendReversalAnalysisSet {
            macps,
            macoses,
            ecp2s,
            msesps,
        } = value;

        let macps = Self::resolve_latest_signal(macps.buy, macps.sell);
        let macos_close_golden = macoses.latest_based_on_close(&GoldenCross);
        let macos_close_dead = macoses.latest_based_on_close(&DeadCross);
        let macos = Self::resolve_latest_signal(macos_close_golden, macos_close_dead);
        let ecp2_buy = Self::latest_buy_sell_signal_date(ecp2s, &Buy);
        let ecp2_sell = Self::latest_buy_sell_signal_date(ecp2s, &Sell);
        let ecp2 = Self::resolve_latest_signal(ecp2_buy, ecp2_sell);
        let msesp_buy = Self::latest_buy_sell_signal_date(msesps, &Buy);
        let msesp_sell = Self::latest_buy_sell_signal_date(msesps, &Sell);
        let msesp = Self::resolve_latest_signal(msesp_buy, msesp_sell);
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
    use anyhow::Result;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::ecp2::model::ECP2s;
    use crate::domain::models::macos::model::MACOSes;
    use crate::domain::models::macps::model::MACPS;
    use crate::domain::models::msesp::model::MSESPes;
    use crate::domain::models::sma::model::{SMAListPair, SMAListTrio, SMAs};
    use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
    use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysisSet;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const MARUBOZU_MIN_RATE: usize = 90;
    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn trend_reversal_analysis_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(query).await?;
        let candle_sticks = CandleSticks::<MARUBOZU_MIN_RATE>::try_from(stocks.as_slice())?;
        let ecp2s = ECP2s::from(candle_sticks.as_slice());
        let msespes = MSESPes::from(candle_sticks.as_slice());
        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());
        let smas_50 = SMAs::<50>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let macoses = MACOSes::from(sma_list_pair);
        let sma_list_trio = SMAListTrio {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
            smas_p: smas_50.as_slice(),
        };
        let macps = MACPS::from((stocks.as_slice(), sma_list_trio));
        let candle_stick_pattern_set = TrendReversalAnalysisSet {
            ecp2s: &ecp2s,
            msesps: &msespes,
            macps: &macps,
            macoses: &macoses,
        };
        let actual = TrendReversalAnalysis::from(candle_stick_pattern_set);
        let expected = TrendReversalAnalysis {
            r#type: Stay,
            macos: NaiveDate::from_ymd_opt(2023, 8, 30).map(|date| (date, Buy)),
            ecp2: NaiveDate::from_ymd_opt(2023, 8, 14).map(|date| (date, Sell)),
            msesp: NaiveDate::from_ymd_opt(2023, 8, 31).map(|date| (date, Buy)),
            macps: NaiveDate::from_ymd_opt(2023, 9, 8).map(|date| (date, Sell)),
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
