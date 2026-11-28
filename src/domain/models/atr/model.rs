use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use fin_primitives::signals::batch_f64::{AtrSeed, atr};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct ATR {
    pub date: NaiveDate,
    pub period: u32,
    pub value: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct ATRs<const PERIOD: usize>(pub Vec<ATR>);

impl<const PERIOD: usize> From<&[Stock]> for ATRs<PERIOD> {
    fn from(value: &[Stock]) -> Self {
        const {
            assert!(PERIOD > 0, "ATR period must be > 0");
        }
        if value.len() < PERIOD + 1 {
            return Self(vec![]);
        }

        let high: Vec<f64> = value.iter().map(|stock| stock.adj_high).collect();
        let low: Vec<f64> = value.iter().map(|stock| stock.adj_low).collect();
        let close: Vec<f64> = value.iter().map(|stock| stock.adj_close).collect();
        let atrs = atr(&high, &low, &close, PERIOD, AtrSeed::TaLib)
            .into_iter()
            .zip(value)
            .skip(PERIOD)
            .map(|(atr_value, stock)| ATR {
                date: stock.date,
                period: PERIOD as u32,
                value: atr_value,
            })
            .collect();

        Self(atrs)
    }
}

impl<const PERIOD: usize> From<&ATRs<PERIOD>> for Vec<DerivedFact> {
    fn from(value: &ATRs<PERIOD>) -> Self {
        value
            .iter()
            .map(|atr| DerivedFact {
                metric: MetricKind::Atr,
                date: atr.date,
                value: atr.value,
                period: Some(atr.period),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ATRs;
    use crate::domain::models::stock::model::Stock;
    use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    fn stock(date: NaiveDate, high: f64, low: f64, close: f64) -> Stock {
        Stock {
            date,
            high,
            low,
            close,
            adj_high: high,
            adj_low: low,
            adj_close: close,
            ..Default::default()
        }
    }

    #[test]
    fn returns_empty_for_short_input() {
        let stocks = (0..14)
            .map(|i| {
                let close = 100.0 + i as f64;
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    close + 1.0,
                    close - 1.0,
                    close,
                )
            })
            .collect::<Vec<_>>();

        let atrs = ATRs::<14>::from(stocks.as_slice());

        assert!(atrs.is_empty());
    }

    #[test]
    fn atr_values_are_non_negative() {
        let stocks = (0..20)
            .map(|i| {
                let close = 100.0 + (i as f64 * 0.5);
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    close + 2.0,
                    close - 1.5,
                    close,
                )
            })
            .collect::<Vec<_>>();

        let atrs = ATRs::<14>::from(stocks.as_slice());

        assert_eq!(atrs.len(), 6);
        assert!(atrs.iter().all(|atr| atr.value >= 0.0));
    }

    #[test]
    fn converts_to_derived_facts() {
        let stocks = (0..20)
            .map(|i| {
                let close = 120.0 - (i as f64 * 0.3);
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    close + 1.8,
                    close - 1.2,
                    close,
                )
            })
            .collect::<Vec<_>>();

        let atrs = ATRs::<14>::from(stocks.as_slice());
        let facts: Vec<DerivedFact> = Vec::from(&atrs);

        assert_eq!(facts.len(), atrs.len());
        assert!(facts.iter().all(|fact| fact.metric == MetricKind::Atr));
        assert!(facts.iter().all(|fact| fact.period == Some(14)));
        assert!(
            facts
                .iter()
                .all(|fact| (0.0..=f64::MAX).contains(&fact.value))
        );
    }

    #[test]
    fn uses_wilder_smoothing() {
        // 15 bars with a true range of 1.0, then one bar with a true range of 3.0.
        let mut stocks = (0..16)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.5,
                    99.5,
                    100.0,
                )
            })
            .collect::<Vec<_>>();
        stocks[15].adj_high = 101.5;
        stocks[15].adj_low = 98.5;

        let atrs = ATRs::<14>::from(stocks.as_slice());

        assert_eq!(atrs.len(), 2);
        assert!((atrs[0].value - 1.0).abs() < 1e-9);
        // Wilder: (13 * 1.0 + 3.0) / 14
        assert!(
            (atrs[1].value - 16.0 / 14.0).abs() < 1e-9,
            "{}",
            atrs[1].value
        );
    }
}
