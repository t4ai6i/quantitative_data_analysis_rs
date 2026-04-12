use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern::{
    DeadCross, GoldenCross, Neither,
};
use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct LatestChance {
    pub golden_cross: Option<NaiveDate>,
    pub dead_cross: Option<NaiveDate>,
}

impl From<LatestChance> for CrossoverPattern {
    fn from(value: LatestChance) -> Self {
        let LatestChance {
            golden_cross: latest_golden_chance,
            dead_cross: latest_dead_chance,
        } = value;
        match (latest_golden_chance, latest_dead_chance) {
            (Some(golden), Some(dead)) => {
                if golden >= dead {
                    GoldenCross
                } else {
                    DeadCross
                }
            }
            (Some(_), None) => GoldenCross,
            (None, Some(_)) => DeadCross,
            _ => Neither,
        }
    }
}

impl From<LatestChance> for NaiveDate {
    fn from(value: LatestChance) -> Self {
        let LatestChance {
            golden_cross: latest_golden_chance,
            dead_cross: latest_dead_chance,
        } = value;
        match (latest_golden_chance, latest_dead_chance) {
            (Some(golden), Some(dead)) => {
                if golden >= dead {
                    golden
                } else {
                    dead
                }
            }
            (Some(golden), None) => golden,
            (None, Some(dead)) => dead,
            _ => NaiveDate::default(),
        }
    }
}
