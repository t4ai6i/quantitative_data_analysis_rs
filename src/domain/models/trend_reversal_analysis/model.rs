use rayon::prelude::*;

use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern::{
    DeadCross, GoldenCross,
};
use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
use crate::domain::models::crossover_strategy::sma_cps::model::SmaCps;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// SmaCos/SmaCps/BodyEngulfing/MsEsの売買シグナルから相場転換を分析する
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct TrendReversalAnalysis {
    pub r#type: BuySellSignalType,
    pub sma_cos: Option<(NaiveDate, BuySellSignalType)>,
    pub sma_cps: Option<(NaiveDate, BuySellSignalType)>,
    pub body_engulfing: Option<(NaiveDate, BuySellSignalType)>,
    pub ms_es: Option<(NaiveDate, BuySellSignalType)>,
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
    pub body_engulfings: &'a [BuySellSignal],
    pub ms_eses: &'a [BuySellSignal],
    pub sma_cps: &'a SmaCps,
    pub sma_coses: &'a SmaCoses,
}

impl<'a> From<TrendReversalAnalysisSet<'a>> for TrendReversalAnalysis {
    fn from(value: TrendReversalAnalysisSet<'a>) -> Self {
        let TrendReversalAnalysisSet {
            sma_cps,
            sma_coses,
            body_engulfings,
            ms_eses,
        } = value;

        let sma_cps = Self::resolve_latest_signal(sma_cps.buy, sma_cps.sell);
        let sma_cos_close_golden = sma_coses.latest_based_on_close(&GoldenCross);
        let sma_cos_close_dead = sma_coses.latest_based_on_close(&DeadCross);
        let sma_cos = Self::resolve_latest_signal(sma_cos_close_golden, sma_cos_close_dead);
        let body_engulfing_buy = Self::latest_buy_sell_signal_date(body_engulfings, &Buy);
        let body_engulfing_sell = Self::latest_buy_sell_signal_date(body_engulfings, &Sell);
        let body_engulfing = Self::resolve_latest_signal(body_engulfing_buy, body_engulfing_sell);
        let ms_es_buy = Self::latest_buy_sell_signal_date(ms_eses, &Buy);
        let ms_es_sell = Self::latest_buy_sell_signal_date(ms_eses, &Sell);
        let ms_es = Self::resolve_latest_signal(ms_es_buy, ms_es_sell);
        // 終値のSmaCos GoldenCross/BodyEngulfing Buy/MsEs Buy/SmaCps Buy が揃っていれば Buy
        // 終値のSmaCos DeadCross/BodyEngulfing Sell/MsEs Sell/SmaCps Sell が揃っていれば Sell
        // それ以外はStay
        let r#type = match (sma_cos, sma_cps, body_engulfing, ms_es) {
            (Some(sma_cos), Some(sma_cps), Some(body_engulfing), Some(ms_es))
                if sma_cos.1.eq(&Buy)
                    && sma_cps.1.eq(&Buy)
                    && body_engulfing.1.eq(&Buy)
                    && ms_es.1.eq(&Buy) =>
            {
                Buy
            }
            (Some(sma_cos), Some(sma_cps), Some(body_engulfing), Some(ms_es))
                if sma_cos.1.eq(&Sell)
                    && sma_cps.1.eq(&Sell)
                    && body_engulfing.1.eq(&Sell)
                    && ms_es.1.eq(&Sell) =>
            {
                Sell
            }
            _ => Stay,
        };
        Self {
            r#type,
            sma_cps,
            sma_cos,
            body_engulfing,
            ms_es,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    use crate::domain::models::body_engulfing::model::BodyEngulfings;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
    use crate::domain::models::crossover_strategy::sma_cps::model::SmaCps;
    use crate::domain::models::ms_es::model::MsEses;
    use crate::domain::models::sma::model::{SMAListPair, SMAListTrio, SMAs};
    use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
    use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysisSet;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;

    const MARUBOZU_BODY_MIN_RATIO: usize = 90;
    const MARUBOZU_WICK_MAX_RATIO: usize = 2;
    const DOJI_MAX_BODY_RATIO: usize = 5;
    const CSV: &[u8] = include_bytes!("../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn trend_reversal_analysis_test() -> Result<()> {
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
        let ms_eses = MsEses::from(candle_sticks.as_slice());
        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());
        let smas_50 = SMAs::<50>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let sma_coses = SmaCoses::from(sma_list_pair);
        let sma_list_trio = SMAListTrio {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
            smas_p: smas_50.as_slice(),
        };
        let sma_cps = SmaCps::from((stocks.as_slice(), sma_list_trio));
        let trend_reversal_analysis_set = TrendReversalAnalysisSet {
            body_engulfings: &body_engulfings,
            ms_eses: &ms_eses,
            sma_cps: &sma_cps,
            sma_coses: &sma_coses,
        };
        let actual = TrendReversalAnalysis::from(trend_reversal_analysis_set);
        let expected = TrendReversalAnalysis {
            r#type: Stay,
            sma_cos: NaiveDate::from_ymd_opt(2023, 8, 30).map(|date| (date, Buy)),
            body_engulfing: NaiveDate::from_ymd_opt(2023, 8, 14).map(|date| (date, Sell)),
            ms_es: NaiveDate::from_ymd_opt(2023, 5, 24).map(|date| (date, Sell)),
            sma_cps: NaiveDate::from_ymd_opt(2023, 9, 8).map(|date| (date, Sell)),
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
