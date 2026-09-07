use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use ta::Next;
use ta::indicators::RelativeStrengthIndex;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct RSI {
    pub date: NaiveDate,
    pub period: u32,
    pub value: f64,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Default, Deref, DerefMut)]
pub struct RSIs<const PERIOD: usize>(pub Vec<RSI>);

impl<const PERIOD: usize> From<&[Stock]> for RSIs<PERIOD> {
    fn from(value: &[Stock]) -> Self {
        const {
            assert!(PERIOD > 0, "RSI period must be > 0");
        }
        if value.len() < PERIOD + 1 {
            return Self(vec![]);
        }

        // use unwrap because we have already checked that PERIOD > 0
        let mut rsi = RelativeStrengthIndex::new(PERIOD).unwrap();
        let mut rsis = Vec::with_capacity(value.len() - PERIOD);
        for (index, stock) in value.iter().enumerate() {
            let rsi_value = rsi.next(stock.close);
            if index < PERIOD {
                continue;
            }

            rsis.push(RSI {
                date: stock.date,
                period: PERIOD as u32,
                value: rsi_value,
            });
        }
        Self(rsis)
    }
}

impl<const PERIOD: usize> From<&RSIs<PERIOD>> for Vec<DerivedFact> {
    fn from(value: &RSIs<PERIOD>) -> Self {
        value
            .iter()
            .map(|rsi| DerivedFact {
                metric: MetricKind::Rsi,
                date: rsi.date,
                value: rsi.value,
                period: Some(rsi.period),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::RSIs;
    use crate::domain::models::stock::model::Stock;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    fn stock(date: NaiveDate, close: f64) -> Stock {
        Stock {
            date,
            close,
            ..Default::default()
        }
    }

    #[test]
    fn returns_empty_for_short_input() {
        let stocks = (0..14)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0,
                )
            })
            .collect::<Vec<_>>();

        let rsis = RSIs::<14>::from(stocks.as_slice());

        assert!(rsis.is_empty());
    }

    #[test]
    fn increasing_series_reports_100_rsi() {
        let stocks = (0..20)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0 + i as f64,
                )
            })
            .collect::<Vec<_>>();

        let rsis = RSIs::<14>::from(stocks.as_slice());

        assert_eq!(rsis.len(), 6);
        let last = rsis.last().unwrap().value;
        assert!(last > 70.0);
        assert!(last <= 100.0);
    }

    #[test]
    fn decreasing_series_reports_0_rsi() {
        let stocks = (0..20)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    200.0 - i as f64,
                )
            })
            .collect::<Vec<_>>();

        let rsis = RSIs::<14>::from(stocks.as_slice());

        assert_eq!(rsis.len(), 6);
        let last = rsis.last().unwrap().value;
        assert!(last < 30.0);
        assert!(last >= 0.0);
    }

    #[test]
    fn rsi_values_are_bounded_between_0_and_100() {
        let closes = [
            100.0, 101.0, 99.0, 102.0, 98.0, 103.0, 97.0, 104.0, 96.0, 105.0, 95.0, 106.0, 94.0,
            107.0, 93.0, 108.0, 92.0, 109.0, 91.0, 110.0,
        ];
        let stocks = closes
            .iter()
            .enumerate()
            .map(|(i, close)| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    *close,
                )
            })
            .collect::<Vec<_>>();

        let rsis = RSIs::<14>::from(stocks.as_slice());

        assert_eq!(rsis.len(), 6);
        assert!(rsis.iter().all(|rsi| (0.0..=100.0).contains(&rsi.value)));
    }
}
