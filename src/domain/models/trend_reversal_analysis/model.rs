use rayon::prelude::*;

use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
use crate::domain::models::buy_sell_signal::model::{BuySellSignal, BuySellSignalType};
use crate::domain::models::crossover_strategy::crossover_pattern;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern::{
    DeadCross, GoldenCross,
};
use crate::domain::models::crossover_strategy::macd_cos::model::{MacdCos, MacdCoses};
use crate::domain::models::crossover_strategy::sma_cos::model::{SmaCos, SmaCoses};
use crate::domain::models::crossover_strategy::sma_cps::model::SmaCps;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// SmaCos/SmaCps/BodyEngulfing/MsEs/MacdCosの売買シグナルから相場転換を分析する
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct TrendReversalAnalysis {
    pub r#type: BuySellSignalType,
    pub sma_cos: Option<(NaiveDate, BuySellSignalType)>,
    pub sma_cps: Option<(NaiveDate, BuySellSignalType)>,
    pub body_engulfing: Option<(NaiveDate, BuySellSignalType)>,
    pub ms_es: Option<(NaiveDate, BuySellSignalType)>,
    pub macd_cos: Option<(NaiveDate, BuySellSignalType)>,
}

fn all_signals_match(
    signal0: Option<(NaiveDate, BuySellSignalType)>,
    signal1: Option<(NaiveDate, BuySellSignalType)>,
    signal2: Option<(NaiveDate, BuySellSignalType)>,
    signal3: Option<(NaiveDate, BuySellSignalType)>,
    signal4: Option<(NaiveDate, BuySellSignalType)>,
    target: BuySellSignalType,
) -> bool {
    signal0.is_some_and(|s| s.1 == target)
        && signal1.is_some_and(|s| s.1 == target)
        && signal2.is_some_and(|s| s.1 == target)
        && signal3.is_some_and(|s| s.1 == target)
        && signal4.is_some_and(|s| s.1 == target)
}

fn latest_buy_or_sell_signal_type(
    buy_sell_signal: &[BuySellSignal],
) -> Option<(NaiveDate, BuySellSignalType)> {
    buy_sell_signal
        .par_iter()
        .filter_map(|signal| {
            matches!(signal.r#type, Buy | Sell).then_some((signal.date, signal.r#type))
        })
        .max_by(|(a, _), (b, _)| Ord::cmp(a, b))
}

fn latest_signal_from_buy_sell_dates(
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

fn crossover_pattern_to_buy_sell_signal(
    date: NaiveDate,
    crossover_pattern: CrossoverPattern,
) -> Option<(NaiveDate, BuySellSignalType)> {
    matches!(crossover_pattern, GoldenCross | DeadCross)
        .then_some((date, crossover_pattern))
        .map(|(date, crossover_pattern)| (date, BuySellSignalType::from(crossover_pattern)))
}

pub struct TrendReversalAnalysisSet<'a> {
    pub body_engulfings: &'a [BuySellSignal],
    pub ms_eses: &'a [BuySellSignal],
    pub sma_cps: &'a SmaCps,
    pub sma_coses: &'a SmaCoses,
    pub macd_coses: &'a MacdCoses,
}

impl<'a> From<TrendReversalAnalysisSet<'a>> for TrendReversalAnalysis {
    fn from(value: TrendReversalAnalysisSet<'a>) -> Self {
        let TrendReversalAnalysisSet {
            sma_cps,
            sma_coses,
            body_engulfings,
            ms_eses,
            macd_coses,
        } = value;

        let sma_cos = sma_coses
            .par_iter()
            .filter_map(
                |&SmaCos {
                     date,
                     crossover_pattern_close,
                     ..
                 }| {
                    let crossover_pattern::close::model::CrossoverPattern(crossover_pattern) =
                        crossover_pattern_close;
                    crossover_pattern_to_buy_sell_signal(date, crossover_pattern)
                },
            )
            .max_by(|(a, _), (b, _)| Ord::cmp(a, b));
        let macd_cos = macd_coses
            .par_iter()
            .filter_map(
                |&MacdCos {
                     date,
                     crossover_pattern,
                     ..
                 }| crossover_pattern_to_buy_sell_signal(date, crossover_pattern),
            )
            .max_by(|(a, _), (b, _)| Ord::cmp(a, b));
        let sma_cps = latest_signal_from_buy_sell_dates(sma_cps.buy, sma_cps.sell);
        let body_engulfing = latest_buy_or_sell_signal_type(body_engulfings);
        let ms_es = latest_buy_or_sell_signal_type(ms_eses);
        // 終値のSmaCos GoldenCross/MacdCos GoldenCross/BodyEngulfing Buy/MsEs Buy/SmaCps Buy が揃っていれば Buy
        // 終値のSmaCos DeadCross/MacdCos DeadCross/BodyEngulfing Sell/MsEs Sell/SmaCps Sell が揃っていれば Sell
        // それ以外はStay
        let r#type = if all_signals_match(sma_cos, macd_cos, sma_cps, body_engulfing, ms_es, Buy) {
            Buy
        } else if all_signals_match(sma_cos, macd_cos, sma_cps, body_engulfing, ms_es, Sell) {
            Sell
        } else {
            Stay
        };
        Self {
            r#type,
            sma_cps,
            sma_cos,
            body_engulfing,
            ms_es,
            macd_cos,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::body_engulfing::model::BodyEngulfings;
    use crate::domain::models::buy_sell_signal::model::BuySellSignalType::{Buy, Sell, Stay};
    use crate::domain::models::candle_stick::model::CandleSticks;
    use crate::domain::models::crossover_strategy::macd_cos::model::MacdCoses;
    use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
    use crate::domain::models::crossover_strategy::sma_cps::model::SmaCps;
    use crate::domain::models::macd::model::MACDs;
    use crate::domain::models::ms_es::model::MsEses;
    use crate::domain::models::sma::model::{SMAListPair, SMAListTrio, SMAs};
    use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysis;
    use crate::domain::models::trend_reversal_analysis::model::TrendReversalAnalysisSet;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;
    use anyhow::Result;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    const MARUBOZU_BODY_MIN_RATIO: usize = 90;
    const MARUBOZU_WICK_MAX_RATIO: usize = 2;
    const DOJI_MAX_BODY_RATIO: usize = 5;
    const FAST_PERIOD: usize = 12;
    const SLOW_PERIOD: usize = 26;
    const SIGNAL_PERIOD: usize = 9;
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
        let macds = MACDs::<FAST_PERIOD, SLOW_PERIOD, SIGNAL_PERIOD>::from(stocks.as_slice());
        let macd_coses = MacdCoses::from(macds);
        let trend_reversal_analysis_set = TrendReversalAnalysisSet {
            body_engulfings: &body_engulfings,
            ms_eses: &ms_eses,
            sma_cps: &sma_cps,
            sma_coses: &sma_coses,
            macd_coses: &macd_coses,
        };
        let actual = TrendReversalAnalysis::from(trend_reversal_analysis_set);
        let expected = TrendReversalAnalysis {
            r#type: Stay,
            sma_cos: NaiveDate::from_ymd_opt(2023, 8, 30).map(|date| (date, Buy)),
            body_engulfing: NaiveDate::from_ymd_opt(2023, 8, 14).map(|date| (date, Sell)),
            ms_es: NaiveDate::from_ymd_opt(2023, 5, 24).map(|date| (date, Sell)),
            sma_cps: NaiveDate::from_ymd_opt(2023, 9, 8).map(|date| (date, Sell)),
            macd_cos: NaiveDate::from_ymd_opt(2023, 8, 30).map(|date| (date, Buy)),
        };
        assert_eq!(actual, expected);
        Ok(())
    }
}
