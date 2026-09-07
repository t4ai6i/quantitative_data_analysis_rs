use crate::domain::models::stock::model::Stock;
use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use ta::Next;
use ta::indicators::SimpleMovingAverage;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct VolumeSpike {
    pub date: NaiveDate,
    pub period: u32,
    pub value: f64,
}

#[derive(Debug, Clone, PartialOrd, PartialEq, Default, Deref, DerefMut)]
pub struct VolumeSpikes<const PERIOD: usize>(pub Vec<VolumeSpike>);

impl<const PERIOD: usize> From<&[Stock]> for VolumeSpikes<PERIOD> {
    fn from(value: &[Stock]) -> Self {
        const {
            assert!(PERIOD > 0, "Volume Spike period must be > 0");
        }
        if value.len() < PERIOD + 1 {
            return Self(vec![]);
        }

        // use unwrap because PERIOD > 0 is asserted at compile time
        let mut sma = SimpleMovingAverage::new(PERIOD).unwrap();
        let mut volume_spikes = Vec::with_capacity(value.len() - PERIOD);

        for (index, stock) in value.iter().enumerate() {
            if index < PERIOD {
                sma.next(stock.volume as f64);
                continue;
            }

            // average of previous PERIOD bars (current bar is excluded)
            let average_volume = sma.next(value[index - 1].volume as f64);
            let ratio = if average_volume > 0.0 {
                stock.volume as f64 / average_volume
            } else {
                0.0
            };

            volume_spikes.push(VolumeSpike {
                date: stock.date,
                period: PERIOD as u32,
                value: ratio,
            });
        }

        Self(volume_spikes)
    }
}

impl<const PERIOD: usize> From<&VolumeSpikes<PERIOD>> for Vec<DerivedFact> {
    fn from(value: &VolumeSpikes<PERIOD>) -> Self {
        value
            .iter()
            .map(|volume_spike| DerivedFact {
                metric: MetricKind::VolumeRatio,
                date: volume_spike.date,
                value: volume_spike.value,
                period: Some(volume_spike.period),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::VolumeSpikes;
    use crate::domain::models::stock::model::Stock;
    use crate::domain::models::technical_analysis::model::{DerivedFact, MetricKind};
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    fn stock(date: NaiveDate, volume: u64) -> Stock {
        Stock {
            date,
            volume,
            ..Default::default()
        }
    }

    #[test]
    fn returns_empty_for_short_input() {
        let stocks = (0..14)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    1_000,
                )
            })
            .collect::<Vec<_>>();

        let spikes = VolumeSpikes::<14>::from(stocks.as_slice());

        assert!(spikes.is_empty());
    }

    #[test]
    fn computes_ratio_against_previous_period_average() {
        let mut stocks = (0..14)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    1_000,
                )
            })
            .collect::<Vec<_>>();
        stocks.push(stock(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), 2_000));

        let spikes = VolumeSpikes::<14>::from(stocks.as_slice());

        assert_eq!(spikes.len(), 1);
        assert!((spikes[0].value - 2.0).abs() < 1e-10);
    }

    #[test]
    fn converts_to_derived_facts() {
        let mut stocks = (0..14)
            .map(|i| {
                stock(
                    NaiveDate::from_ymd_opt(2024, 1, 1 + i as u32).unwrap(),
                    1_000,
                )
            })
            .collect::<Vec<_>>();
        stocks.push(stock(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(), 1_500));
        stocks.push(stock(NaiveDate::from_ymd_opt(2024, 1, 16).unwrap(), 500));

        let spikes = VolumeSpikes::<14>::from(stocks.as_slice());
        let facts: Vec<DerivedFact> = Vec::from(&spikes);

        assert_eq!(facts.len(), spikes.len());
        assert!(
            facts
                .iter()
                .all(|fact| fact.metric == MetricKind::VolumeRatio)
        );
        assert!(facts.iter().all(|fact| fact.period == Some(14)));
        assert!(facts.iter().all(|fact| fact.value >= 0.0));
    }
}
