use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use ta::Next;
use ta::indicators::BollingerBands;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Bollinger {
    pub date: NaiveDate,
    pub period: u32,
    pub middle: f64, // SMA
    pub upper: f64,  // middle + 2σ
    pub lower: f64,  // middle - 2σ
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Default, Deref, DerefMut)]
pub struct Bollingers<const PERIOD: usize>(pub Vec<Bollinger>);

impl<const PERIOD: usize> From<&[Stock]> for Bollingers<PERIOD> {
    fn from(value: &[Stock]) -> Self {
        const {
            assert!(PERIOD > 0, "Bollinger period must be > 0");
        }
        if value.len() < PERIOD {
            return Self(vec![]);
        }

        // Use unwrap because PERIOD > 0 is asserted at compile time.
        let mut bb = BollingerBands::new(PERIOD, 2.0).unwrap();
        let mut bollingers = Vec::with_capacity(value.len() - PERIOD + 1);
        for (index, stock) in value.iter().enumerate() {
            let output = bb.next(stock.close);
            if index < PERIOD - 1 {
                continue;
            }

            bollingers.push(Bollinger {
                date: stock.date,
                period: PERIOD as u32,
                middle: output.average,
                upper: output.upper,
                lower: output.lower,
            });
        }
        Self(bollingers)
    }
}

impl<const PERIOD: usize> From<&Bollingers<PERIOD>> for Vec<DerivedFact> {
    fn from(value: &Bollingers<PERIOD>) -> Self {
        value
            .iter()
            .flat_map(|band| {
                [
                    DerivedFact {
                        metric: MetricKind::BollingerUpper,
                        date: band.date,
                        value: band.upper,
                        period: Some(band.period),
                    },
                    DerivedFact {
                        metric: MetricKind::BollingerLower,
                        date: band.date,
                        value: band.lower,
                        period: Some(band.period),
                    },
                ]
            })
            .collect()
    }
}
#[cfg(test)]
mod tests {
    use super::Bollingers;
    use crate::domain::models::stock::model::Stock;
    use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
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
        let stocks = (0..19)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0,
                )
            })
            .collect::<Vec<_>>();

        let result = Bollingers::<20>::from(stocks.as_slice());

        assert!(result.is_empty());
    }

    #[test]
    fn constant_series_has_zero_width_band() {
        let stocks = (0..25)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0,
                )
            })
            .collect::<Vec<_>>();

        let result = Bollingers::<20>::from(stocks.as_slice());

        assert_eq!(result.len(), 6);
        assert!(result.iter().all(|b| (b.middle - 100.0).abs() < 1e-10));
        assert!(result.iter().all(|b| (b.upper - 100.0).abs() < 1e-10));
        assert!(result.iter().all(|b| (b.lower - 100.0).abs() < 1e-10));
    }

    #[test]
    fn converts_to_derived_facts_as_upper_and_lower_pairs() {
        let stocks = (0..25)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0 + i as f64,
                )
            })
            .collect::<Vec<_>>();

        let bollingers = Bollingers::<20>::from(stocks.as_slice());
        let facts: Vec<DerivedFact> = Vec::from(&bollingers);

        assert_eq!(facts.len(), bollingers.len() * 2);

        for pair in facts.chunks(2) {
            assert_eq!(pair[0].metric, MetricKind::BollingerUpper);
            assert_eq!(pair[1].metric, MetricKind::BollingerLower);
            assert_eq!(pair[0].date, pair[1].date);
            assert_eq!(pair[0].period, pair[1].period);
            assert!(pair[0].value >= pair[1].value);
        }
    }

    #[test]
    fn bollinger_values_follow_general_band_constraints() {
        let stocks = (0..25)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    100.0 + (i as f64 / 3.0),
                )
            })
            .collect::<Vec<_>>();

        let result = Bollingers::<20>::from(stocks.as_slice());

        assert_eq!(result.len(), 6);
        assert!(
            result
                .iter()
                .all(|b| b.upper >= b.middle && b.middle >= b.lower)
        );
    }
}
