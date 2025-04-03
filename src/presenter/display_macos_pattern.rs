use chrono::NaiveDate;
use strum::Display;

use crate::domain::models::macos;
use crate::domain::models::macos::model::AnalysisPattern;
use crate::domain::models::macos::model::Pattern::{DeadCross, GoldenCross};
use crate::domain::models::macos_analysis::close::model::{LatestChance, RateOfChance};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
pub enum DisplayMACOSPattern {
    #[default]
    All,
    GoldenOnly,
    DeadOnly,
}

/// View向けMovingAverageCrossoverStrategyパターン
impl DisplayMACOSPattern {
    /// 指定されたMACOSPatternと比較
    pub fn is_display_by_macos_pattern(&self, pattern: &macos::model::Pattern) -> bool {
        match self {
            DisplayMACOSPattern::All => true,
            DisplayMACOSPattern::GoldenOnly => pattern.eq(&GoldenCross),
            DisplayMACOSPattern::DeadOnly => pattern.eq(&DeadCross),
        }
    }

    pub fn get_rate_of_chance(self, rate_of_chance: &RateOfChance) -> f64 {
        match self {
            DisplayMACOSPattern::All => rate_of_chance.whole,
            DisplayMACOSPattern::GoldenOnly => rate_of_chance.golden,
            DisplayMACOSPattern::DeadOnly => rate_of_chance.dead,
        }
    }

    pub fn get_latest_chance(self, latest_chance: &LatestChance) -> LatestChance {
        match self {
            DisplayMACOSPattern::All => *latest_chance,
            DisplayMACOSPattern::GoldenOnly => LatestChance {
                golden_cross: latest_chance.golden_cross,
                dead_cross: None,
            },
            DisplayMACOSPattern::DeadOnly => LatestChance {
                golden_cross: None,
                dead_cross: latest_chance.dead_cross,
            },
        }
    }

    pub fn latest_chance_within_days(
        self,
        within_days: NaiveDate,
        latest_chance: &LatestChance,
    ) -> Option<(AnalysisPattern, NaiveDate)> {
        let LatestChance {
            golden_cross: latest_golden_chance,
            dead_cross: latest_dead_chance,
        } = latest_chance;
        match self {
            DisplayMACOSPattern::All => {
                if latest_golden_chance.is_none() || latest_dead_chance.is_none() {
                    return None;
                };
                let latest_golden_chance = latest_golden_chance.unwrap();
                let latest_dead_chance = latest_dead_chance.unwrap();
                if latest_golden_chance >= latest_dead_chance {
                    if latest_golden_chance >= within_days {
                        Some((AnalysisPattern::GoldenChance, latest_golden_chance))
                    } else {
                        None
                    }
                } else if latest_dead_chance >= within_days {
                    Some((AnalysisPattern::DeadChance, latest_dead_chance))
                } else {
                    None
                }
            }
            DisplayMACOSPattern::GoldenOnly => {
                if latest_golden_chance.is_none() {
                    return None;
                };
                let latest_golden_chance = latest_golden_chance.unwrap();
                if latest_golden_chance >= within_days {
                    Some((AnalysisPattern::GoldenChance, latest_golden_chance))
                } else {
                    None
                }
            }
            DisplayMACOSPattern::DeadOnly => {
                if latest_dead_chance.is_none() {
                    return None;
                };
                let latest_dead_chance = latest_dead_chance.unwrap();
                if latest_dead_chance >= within_days {
                    Some((AnalysisPattern::DeadChance, latest_dead_chance))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use chrono::NaiveDate;

    use crate::domain::models::macos_analysis::close::model::{LatestChance, RateOfChance};
    use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

    #[test]
    fn display_macos_direction_test() -> Result<()> {
        let rate_of_chance = RateOfChance {
            whole: 0.0,
            golden: 1.0,
            dead: 2.0,
        };
        let latest_chance = LatestChance {
            golden_cross: NaiveDate::from_ymd_opt(2023, 12, 12),
            dead_cross: NaiveDate::from_ymd_opt(2022, 12, 12),
        };
        let display_macos_pattern = DisplayMACOSPattern::All;
        let actual = display_macos_pattern.get_rate_of_chance(&rate_of_chance);
        assert_eq!(actual, 0.0);
        let actual = display_macos_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                golden_cross: NaiveDate::from_ymd_opt(2023, 12, 12),
                dead_cross: NaiveDate::from_ymd_opt(2022, 12, 12),
            }
        );
        let display_macos_pattern = DisplayMACOSPattern::GoldenOnly;
        let actual = display_macos_pattern.get_rate_of_chance(&rate_of_chance);
        assert_eq!(actual, 1.0);
        let actual = display_macos_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                golden_cross: NaiveDate::from_ymd_opt(2023, 12, 12),
                dead_cross: None,
            }
        );
        let display_macos_pattern = DisplayMACOSPattern::DeadOnly;
        let actual = display_macos_pattern.get_rate_of_chance(&rate_of_chance);
        assert_eq!(actual, 2.0);
        let actual = display_macos_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                golden_cross: None,
                dead_cross: NaiveDate::from_ymd_opt(2022, 12, 12),
            }
        );
        Ok(())
    }
}
