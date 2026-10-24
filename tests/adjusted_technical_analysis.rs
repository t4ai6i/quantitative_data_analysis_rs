use chrono::{Duration, NaiveDate};
use quantitative_data_analysis_rs::domain::models::{
    atr::model::ATRs,
    body_engulfing::model::BodyEngulfings,
    bollinger::model::Bollingers,
    buy_sell_signal::model::BuySellSignalType,
    candle_stick::model::CandleSticks,
    crossover_strategy::{
        macd_cos::model::MacdCoses, sma_cos::model::SmaCoses, sma_cps::model::SmaCps,
    },
    high_low_direction_signal::model::HighLowDirectionSignals,
    macd::model::{MACDs, MacdCrossEvents},
    ms_es::model::MsEses,
    rsi::model::RSIs,
    sma::model::{SMAListPair, SMAListTrio, SMAs, SmaCrossEvents},
    stock::model::{Stock, Stocks},
    technical_analysis::model::EventFact,
    volume_spike::model::VolumeSpikes,
};
use quantitative_data_analysis_rs::presenter::views::stock::view::Stocks as _;

fn bar(day: i64, open: f64, high: f64, low: f64, close: f64, volume: f64) -> Stock {
    Stock {
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() + Duration::days(day),
        open,
        high,
        low,
        close,
        volume: 1_200,
        adj_open: open,
        adj_high: high,
        adj_low: low,
        adj_close: close,
        adj_volume: volume,
    }
}

fn series() -> Vec<Stock> {
    (0..100)
        .map(|i| {
            let close = match i {
                0..30 => 140.0 - i as f64,
                30..65 => 80.0 + (i - 30) as f64 * 2.0,
                _ => 150.0 - (i - 65) as f64 * 2.0,
            };
            let open = close + if i % 2 == 0 { 1.0 } else { -1.0 };
            bar(
                i,
                open,
                close + 3.0,
                close - 3.0,
                close,
                1200.5 + (i % 7) as f64 * 120.0,
            )
        })
        .collect()
}

fn with_corporate_actions(stocks: &[Stock]) -> Vec<Stock> {
    stocks
        .iter()
        .enumerate()
        .map(|(i, stock)| {
            let factor = if i < 30 {
                3.0
            } else if i < 65 {
                0.5
            } else {
                1.0
            };
            Stock {
                open: stock.adj_open * factor,
                high: stock.adj_high * factor,
                low: stock.adj_low * factor,
                close: stock.adj_close * factor,
                volume: if i < 30 {
                    400
                } else if i < 65 {
                    2400
                } else {
                    1200
                },
                ..*stock
            }
        })
        .collect()
}

fn sma_coses(stocks: &[Stock]) -> SmaCoses {
    let fast = SMAs::<5>::from(stocks);
    let slow = SMAs::<25>::from(stocks);
    SmaCoses::from(SMAListPair {
        smas_n: &fast,
        smas_o: &slow,
    })
}

fn sma_cps(stocks: &[Stock]) -> SmaCps {
    let fast = SMAs::<5>::from(stocks);
    let middle = SMAs::<25>::from(stocks);
    let slow = SMAs::<50>::from(stocks);
    SmaCps::from((
        stocks,
        SMAListTrio {
            smas_n: &fast,
            smas_o: &middle,
            smas_p: &slow,
        },
    ))
}

#[test]
fn indicators_and_cross_events_ignore_raw_split_and_consolidation_steps() {
    let baseline = series();
    let actions = with_corporate_actions(&baseline);
    let a = baseline.as_slice();
    let b = actions.as_slice();
    assert_eq!(SMAs::<5>::from(a), SMAs::<5>::from(b));
    assert_eq!(MACDs::<12, 26, 9>::from(a), MACDs::<12, 26, 9>::from(b));
    assert_eq!(RSIs::<14>::from(a), RSIs::<14>::from(b));
    assert_eq!(Bollingers::<20>::from(a), Bollingers::<20>::from(b));
    assert_eq!(ATRs::<14>::from(a), ATRs::<14>::from(b));
    assert_eq!(VolumeSpikes::<14>::from(a), VolumeSpikes::<14>::from(b));
    assert_eq!(sma_coses(a), sma_coses(b));
    assert_eq!(sma_cps(a), sma_cps(b));
    assert!(sma_cps(a).buy.is_some());
    assert!(sma_cps(a).sell.is_some());
    assert_eq!(
        MacdCoses::from(MACDs::<12, 26, 9>::from(a)),
        MacdCoses::from(MACDs::<12, 26, 9>::from(b))
    );
    let events = |stocks: &[Stock]| {
        let macds = MACDs::<12, 26, 9>::from(stocks);
        let mut events: Vec<EventFact> = Vec::from(MacdCrossEvents { macds: &macds });
        let fast = SMAs::<5>::from(stocks);
        let slow = SMAs::<25>::from(stocks);
        events.extend(Vec::<EventFact>::from(SmaCrossEvents {
            sma_list_pair: SMAListPair {
                smas_n: &fast,
                smas_o: &slow,
            },
        }));
        events
    };
    assert!(!events(a).is_empty());
    assert_eq!(events(a), events(b));
}

#[test]
fn candles_and_multi_day_patterns_use_adjusted_prices() {
    let baseline = series();
    let actions = with_corporate_actions(&baseline);
    let a = CandleSticks::<90, 2, 5>::try_from(baseline.as_slice()).unwrap();
    let b = CandleSticks::<90, 2, 5>::try_from(actions.as_slice()).unwrap();
    assert_eq!(a, b);
    assert_eq!(
        BodyEngulfings::from(a.as_slice()),
        BodyEngulfings::from(b.as_slice())
    );
    assert_eq!(MsEses::from(a.as_slice()), MsEses::from(b.as_slice()));
    assert_eq!(
        HighLowDirectionSignals::from(baseline.as_slice()),
        HighLowDirectionSignals::from(actions.as_slice())
    );
}

#[test]
fn engulfing_and_high_low_direction_are_not_reversed_by_a_split() {
    let mut stocks = [
        bar(0, 11.0, 11.5, 9.5, 10.0, 100.5),
        bar(1, 9.0, 12.5, 8.5, 12.0, 200.5),
    ];
    stocks[0].open *= 3.0;
    stocks[0].high *= 3.0;
    stocks[0].low *= 3.0;
    stocks[0].close *= 3.0;
    let candles = CandleSticks::<90, 2, 5>::try_from(stocks.as_slice()).unwrap();
    assert_eq!((candles[0].open, candles[0].close), (11.0, 10.0));
    let engulfings = BodyEngulfings::from(candles.as_slice());
    assert_eq!(engulfings[0].r#type, BuySellSignalType::Buy);
    stocks[1].adj_low = 10.0;
    let directions = HighLowDirectionSignals::from(stocks.as_slice());
    assert_eq!(directions[0].r#type, BuySellSignalType::Buy);
}

#[test]
fn adjusted_candles_feed_morning_and_evening_star_signals() {
    let stocks = [
        bar(0, 11.0, 11.5, 9.5, 10.0, 100.0),
        bar(1, 10.01, 11.0, 9.0, 10.0, 100.0),
        bar(2, 10.0, 11.5, 9.5, 11.0, 100.0),
        bar(3, 11.01, 12.0, 10.0, 11.0, 100.0),
        bar(4, 11.0, 11.5, 9.5, 10.0, 100.0),
    ];
    let mut actions = stocks;
    for stock in &mut actions[..2] {
        stock.open *= 3.0;
        stock.high *= 3.0;
        stock.low *= 3.0;
        stock.close *= 3.0;
    }
    let candles = CandleSticks::<90, 2, 5>::try_from(actions.as_slice()).unwrap();
    let signals = MsEses::from(candles.as_slice());
    assert_eq!(signals[0].r#type, BuySellSignalType::Buy);
    assert_eq!(signals[2].r#type, BuySellSignalType::Sell);
}

#[test]
fn atr_uses_intraday_range_and_previous_adjusted_close() {
    let mut stocks = [
        bar(0, 10.0, 11.0, 9.0, 10.0, 100.0),
        bar(1, 13.0, 14.0, 12.0, 13.0, 100.0),
        bar(2, 10.0, 12.0, 8.0, 9.0, 100.0),
    ];
    stocks[0].close = 30.0;
    // TR = 2, 4, 5; EMA(period=2, alpha=2/3) = 2, 10/3, 40/9.
    let atrs = ATRs::<2>::from(stocks.as_slice());
    assert_eq!(atrs.len(), 1);
    assert!((atrs[0].value - 40.0 / 9.0).abs() < 1e-10);
    assert_eq!(atrs[0].date, stocks[2].date);

    let unchanged_close: Vec<_> = (0..3)
        .map(|i| bar(i, 10.0, 11.0, 9.0, 10.0, 100.0))
        .collect();
    assert_eq!(ATRs::<2>::from(unchanged_close.as_slice())[0].value, 2.0);
}

#[test]
fn volume_ratio_excludes_current_bar_without_repeating_the_previous_bar() {
    let stocks: Vec<_> = [10.5, 20.5, 30.5, 41.0, 82.0]
        .into_iter()
        .enumerate()
        .map(|(i, volume)| bar(i as i64, 10.0, 11.0, 9.0, 10.0, volume))
        .collect();
    let ratios = VolumeSpikes::<3>::from(stocks.as_slice());
    assert_eq!(ratios.len(), 2);
    assert!((ratios[0].value - 2.0).abs() < 1e-10);
    assert!((ratios[1].value - 82.0 / (92.0 / 3.0)).abs() < 1e-10);
    let smas = SMAs::<3>::from(stocks.as_slice());
    assert!((smas[0].volume.0 - 20.5).abs() < 1e-10);
}

#[test]
fn chart_values_share_the_indicator_adjustment_basis() {
    let mut stock = bar(0, 100.0, 104.0, 98.0, 102.0, 200.5);
    stock.open = 300.0;
    stock.high = 312.0;
    stock.low = 294.0;
    stock.close = 306.0;
    stock.volume = 67;
    let stocks = Stocks(vec![stock]);
    assert_eq!(stocks.ohlcs(), vec![100.0, 102.0, 98.0, 104.0]);
    assert_eq!(stocks.volumes(), vec![200.5]);
}
