use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DirectionType {
    Uptrend,
    Downtrend,
    Neutral,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum EventKind {
    GoldenCross,
    DeadCross,
    MorningStar,
    EveningStar,
    BodyEngulfing,
    HighLowDirection,
    VolumeSpike,
    Breakout,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MetricKind {
    Sma,
    Macd,
    Signal,
    Rsi,
    BollingerUpper,
    BollingerLower,
    Atr,
    VolumeRatio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DerivedFact {
    pub metric: MetricKind,
    pub date: NaiveDate,
    pub value: f64,
    pub period: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EventParams {
    Cross {
        fast_metric: MetricKind,
        fast_period: u32,
        slow_metric: MetricKind,
        slow_period: u32,
    },
    Threshold {
        metric: MetricKind,
        threshold: f64,
        direction: DirectionType,
    },
    Pattern {
        pattern_name: EventKind,
        window_bars: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostEventObservation {
    pub close_price_business_day_7: Option<f64>,
    pub close_price_business_day_14: Option<f64>,
    pub close_price_business_day_21: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventFact {
    pub kind: EventKind,
    pub occurred_at: NaiveDate,
    pub direction: DirectionType,
    pub strength: Option<f64>,
    pub event_params: EventParams,
    pub note: Option<String>,
    pub post_event_observation: PostEventObservation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnicalAnalysis {
    pub code: String,
    pub analysis_at: DateTime<Utc>,
    pub derived_facts: Vec<DerivedFact>,
    pub event_facts: Vec<EventFact>,
}
