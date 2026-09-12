use crate::domain::models::atr::model::ATRs;
use crate::domain::models::body_engulfing::model::{BodyEngulfingEvents, BodyEngulfings};
use crate::domain::models::bollinger::model::Bollingers;
use crate::domain::models::candle_stick::model::CandleSticks;
use crate::domain::models::high_low_direction_signal::model::{
    HighLowDirectionEvents, HighLowDirectionSignals,
};
use crate::domain::models::macd::model::{MACDs, MacdCrossEvents};
use crate::domain::models::ms_es::model::{MsEsEvents, MsEses};
use crate::domain::models::rsi::model::RSIs;
use crate::domain::models::sma::model::{SMAListPair, SMAs, SmaCrossEvents};
use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model;
use crate::domain::models::volume_spike::model::VolumeSpikes;
use crate::domain::repositories::stock;
use crate::domain::repositories::stock::queries::get_stocks;
use crate::presenter::presenters::technical_analysis::output;
use crate::use_case::interfaces::technical_analysis::{input, use_case};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TechnicalAnalysis<
    'a,
    SR,
    const MARUBOZU_BODY_MIN_RATIO: usize,
    const MARUBOZU_WICK_MAX_RATIO: usize,
    const DOJI_MAX_BODY_RATIO: usize,
    const SMA_SHORT_PERIOD: usize,
    const SMA_MEDIUM_PERIOD: usize,
    const SMA_LONG_PERIOD: usize,
    const MACD_FAST_PERIOD: usize,
    const MACD_SLOW_PERIOD: usize,
    const MACD_SIGNAL_PERIOD: usize,
    const RSI_PERIOD: usize,
    const BOLLINGER_PERIOD: usize,
    const ATR_PERIOD: usize,
    const VOLUME_SPIKE_PERIOD: usize,
> {
    stock_repository: &'a SR,
}

impl<
    'a,
    SR,
    const MARUBOZU_BODY_MIN_RATIO: usize,
    const MARUBOZU_WICK_MAX_RATIO: usize,
    const DOJI_MAX_BODY_RATIO: usize,
    const SMA_SHORT_PERIOD: usize,
    const SMA_MEDIUM_PERIOD: usize,
    const SMA_LONG_PERIOD: usize,
    const MACD_FAST_PERIOD: usize,
    const MACD_SLOW_PERIOD: usize,
    const MACD_SIGNAL_PERIOD: usize,
    const RSI_PERIOD: usize,
    const BOLLINGER_PERIOD: usize,
    const ATR_PERIOD: usize,
    const VOLUME_SPIKE_PERIOD: usize,
>
    TechnicalAnalysis<
        'a,
        SR,
        MARUBOZU_BODY_MIN_RATIO,
        MARUBOZU_WICK_MAX_RATIO,
        DOJI_MAX_BODY_RATIO,
        SMA_SHORT_PERIOD,
        SMA_MEDIUM_PERIOD,
        SMA_LONG_PERIOD,
        MACD_FAST_PERIOD,
        MACD_SLOW_PERIOD,
        MACD_SIGNAL_PERIOD,
        RSI_PERIOD,
        BOLLINGER_PERIOD,
        ATR_PERIOD,
        VOLUME_SPIKE_PERIOD,
    >
{
    pub fn new(stock_repository: &'a SR) -> Self {
        Self { stock_repository }
    }
}

#[async_trait]
impl<
    SR,
    const MARUBOZU_BODY_MIN_RATIO: usize,
    const MARUBOZU_WICK_MAX_RATIO: usize,
    const DOJI_MAX_BODY_RATIO: usize,
    const SMA_SHORT_PERIOD: usize,
    const SMA_MEDIUM_PERIOD: usize,
    const SMA_LONG_PERIOD: usize,
    const MACD_FAST_PERIOD: usize,
    const MACD_SLOW_PERIOD: usize,
    const MACD_SIGNAL_PERIOD: usize,
    const RSI_PERIOD: usize,
    const BOLLINGER_PERIOD: usize,
    const ATR_PERIOD: usize,
    const VOLUME_SPIKE_PERIOD: usize,
> use_case::TechnicalAnalysis
    for TechnicalAnalysis<
        '_,
        SR,
        MARUBOZU_BODY_MIN_RATIO,
        MARUBOZU_WICK_MAX_RATIO,
        DOJI_MAX_BODY_RATIO,
        SMA_SHORT_PERIOD,
        SMA_MEDIUM_PERIOD,
        SMA_LONG_PERIOD,
        MACD_FAST_PERIOD,
        MACD_SLOW_PERIOD,
        MACD_SIGNAL_PERIOD,
        RSI_PERIOD,
        BOLLINGER_PERIOD,
        ATR_PERIOD,
        VOLUME_SPIKE_PERIOD,
    >
where
    SR: stock::repository::Stock + Send + Sync,
{
    async fn handle(&self, input: input::TechnicalAnalysis) -> Result<output::TechnicalAnalysis> {
        let query = get_stocks::Query {
            code: Some(input.code.as_str()),
            market: None,
            start_date: Some(input.start_date),
            end_date: Some(input.end_date),
        };
        let row_stocks = self.stock_repository.get_stocks(&query).await?;
        let mut stocks = row_stocks.iter().copied().collect::<Vec<Stock>>();
        stocks.sort_by_key(|stock| stock.date);

        let sma_short = SMAs::<SMA_SHORT_PERIOD>::from(stocks.as_slice());
        let sma_medium = SMAs::<SMA_MEDIUM_PERIOD>::from(stocks.as_slice());
        let sma_long = SMAs::<SMA_LONG_PERIOD>::from(stocks.as_slice());
        let macds = MACDs::<MACD_FAST_PERIOD, MACD_SLOW_PERIOD, MACD_SIGNAL_PERIOD>::from(
            stocks.as_slice(),
        );
        let rsis = RSIs::<RSI_PERIOD>::from(stocks.as_slice());
        let bollingers = Bollingers::<BOLLINGER_PERIOD>::from(stocks.as_slice());
        let atrs = ATRs::<ATR_PERIOD>::from(stocks.as_slice());
        let volume_spikes = VolumeSpikes::<VOLUME_SPIKE_PERIOD>::from(stocks.as_slice());
        let candle_sticks = CandleSticks::<
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
        >::try_from(stocks.as_slice())?;
        let body_engulfings = BodyEngulfings::from(candle_sticks.as_slice());
        let ms_eses = MsEses::from(candle_sticks.as_slice());
        let high_low_direction_signals = HighLowDirectionSignals::from(stocks.as_slice());

        let derived_facts = [
            Vec::from(&sma_short),
            Vec::from(&sma_medium),
            Vec::from(&sma_long),
            Vec::from(&macds),
            Vec::from(&rsis),
            Vec::from(&bollingers),
            Vec::from(&atrs),
            Vec::from(&volume_spikes),
        ]
        .concat();

        let event_facts = [
            Vec::from(SmaCrossEvents {
                sma_list_pair: SMAListPair {
                    smas_n: sma_short.as_slice(),
                    smas_o: sma_medium.as_slice(),
                },
            }),
            Vec::from(SmaCrossEvents {
                sma_list_pair: SMAListPair {
                    smas_n: sma_medium.as_slice(),
                    smas_o: sma_long.as_slice(),
                },
            }),
            Vec::from(MacdCrossEvents { macds: &macds }),
            Vec::from(MsEsEvents { mseses: &ms_eses }),
            Vec::from(BodyEngulfingEvents {
                body_engulfings: &body_engulfings,
            }),
            Vec::from(HighLowDirectionEvents {
                signals: &high_low_direction_signals,
            }),
        ]
        .concat();

        Ok(model::TechnicalAnalysis::new(
            input.code,
            input.start_date,
            input.end_date,
            input.analysis_at,
            derived_facts,
            event_facts,
        ))
    }
}
