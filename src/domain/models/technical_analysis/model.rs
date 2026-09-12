use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum DirectionType {
    Uptrend,
    Downtrend,
    Neutral,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum EventKind {
    GoldenCross,
    DeadCross,
    MorningStar,
    EveningStar,
    BullishEngulfing,
    BearishEngulfing,
    HighLowDirection,
    VolumeSpike,
    Breakout,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
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
pub struct EventFact {
    pub kind: EventKind,
    pub occurred_at: NaiveDate,
    pub direction: DirectionType,
    pub event_params: EventParams,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TechnicalAnalysis {
    pub code: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub analysis_at: DateTime<Utc>,
    pub derived_facts: Vec<DerivedFact>,
    pub event_facts: Vec<EventFact>,
}

impl TechnicalAnalysis {
    pub fn new(
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        analysis_at: DateTime<Utc>,
        derived_facts: Vec<DerivedFact>,
        event_facts: Vec<EventFact>,
    ) -> Self {
        Self::from_parts(
            code,
            start_date,
            end_date,
            analysis_at,
            derived_facts,
            event_facts,
        )
    }

    pub fn from_parts(
        code: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        analysis_at: DateTime<Utc>,
        derived_facts: Vec<DerivedFact>,
        event_facts: Vec<EventFact>,
    ) -> Self {
        let mut derived_facts = derived_facts;
        let mut event_facts = event_facts;

        derived_facts.sort_by(|left, right| {
            left.date
                .cmp(&right.date)
                .then(left.metric.cmp(&right.metric))
        });
        event_facts.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then(left.kind.cmp(&right.kind))
        });

        Self {
            code: code.into(),
            start_date,
            end_date,
            analysis_at,
            derived_facts,
            event_facts,
        }
    }

    pub fn push_derived_fact(&mut self, fact: DerivedFact) {
        self.derived_facts.push(fact);
        self.derived_facts.sort_by(|left, right| {
            left.date
                .cmp(&right.date)
                .then(left.metric.cmp(&right.metric))
        });
    }

    pub fn push_event_fact(&mut self, fact: EventFact) {
        self.event_facts.push(fact);
        self.event_facts.sort_by(|left, right| {
            left.occurred_at
                .cmp(&right.occurred_at)
                .then(left.kind.cmp(&right.kind))
        });
    }
}

impl
    From<(
        String,
        NaiveDate,
        NaiveDate,
        DateTime<Utc>,
        Vec<DerivedFact>,
        Vec<EventFact>,
    )> for TechnicalAnalysis
{
    fn from(
        value: (
            String,
            NaiveDate,
            NaiveDate,
            DateTime<Utc>,
            Vec<DerivedFact>,
            Vec<EventFact>,
        ),
    ) -> Self {
        let (code, start_date, end_date, analysis_at, derived_facts, event_facts) = value;
        Self::from_parts(
            code,
            start_date,
            end_date,
            analysis_at,
            derived_facts,
            event_facts,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{DerivedFact, EventFact, EventKind, EventParams, MetricKind, TechnicalAnalysis};
    use chrono::{NaiveDate, TimeZone, Utc};
    use pretty_assertions::assert_eq;

    #[test]
    fn aggregates_and_sorts_facts_by_date() {
        let analysis_at = Utc.with_ymd_and_hms(2024, 1, 31, 9, 30, 0).unwrap();
        let derived_facts = vec![
            DerivedFact {
                metric: MetricKind::Rsi,
                date: NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                value: 70.0,
                period: Some(14),
            },
            DerivedFact {
                metric: MetricKind::Sma,
                date: NaiveDate::from_ymd_opt(2024, 1, 29).unwrap(),
                value: 120.0,
                period: Some(5),
            },
        ];
        let event_facts = vec![
            EventFact {
                kind: EventKind::GoldenCross,
                occurred_at: NaiveDate::from_ymd_opt(2024, 1, 30).unwrap(),
                direction: crate::domain::models::technical_analysis::model::DirectionType::Uptrend,
                event_params: EventParams::Cross {
                    fast_metric: MetricKind::Sma,
                    fast_period: 5,
                    slow_metric: MetricKind::Sma,
                    slow_period: 25,
                },
            },
            EventFact {
                kind: EventKind::DeadCross,
                occurred_at: NaiveDate::from_ymd_opt(2024, 1, 29).unwrap(),
                direction:
                    crate::domain::models::technical_analysis::model::DirectionType::Downtrend,
                event_params: EventParams::Cross {
                    fast_metric: MetricKind::Macd,
                    fast_period: 12,
                    slow_metric: MetricKind::Signal,
                    slow_period: 26,
                },
            },
        ];

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let analysis = TechnicalAnalysis::new(
            "7203",
            start_date,
            end_date,
            analysis_at,
            derived_facts,
            event_facts,
        );

        assert_eq!(
            analysis.derived_facts[0].date,
            NaiveDate::from_ymd_opt(2024, 1, 29).unwrap()
        );
        assert_eq!(
            analysis.event_facts[0].occurred_at,
            NaiveDate::from_ymd_opt(2024, 1, 29).unwrap()
        );
        assert_eq!(analysis.code, "7203");
        assert_eq!(analysis.start_date, start_date);
        assert_eq!(analysis.end_date, end_date);
        assert_eq!(analysis.analysis_at, analysis_at);
    }
}
