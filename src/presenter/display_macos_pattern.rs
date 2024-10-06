use chrono::NaiveDate;
use strum::Display;

use crate::domain::entity::chance_loss::ChanceLoss;
use crate::domain::entity::close_macos_trend_analysis::{ChanceRate, LatestChance};
use crate::domain::entity::macos::MACOSType;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
pub enum DisplayMACOSPattern {
    #[default]
    All,
    GoldenOnly,
    DeadOnly,
}

/// View向けMovingAverageCrossoverStrategyパターン
impl DisplayMACOSPattern {
    /// MACOSTypeによって描画するか判定
    pub fn is_display_by_macos_type(&self, r#type: &MACOSType) -> bool {
        match self {
            DisplayMACOSPattern::All => true,
            DisplayMACOSPattern::GoldenOnly => r#type.eq(&MACOSType::Golden),
            DisplayMACOSPattern::DeadOnly => r#type.eq(&MACOSType::Dead),
        }
    }

    pub fn get_chance_rate(self, chance_rate: &ChanceRate) -> f64 {
        match self {
            DisplayMACOSPattern::All => chance_rate.all,
            DisplayMACOSPattern::GoldenOnly => chance_rate.golden_only,
            DisplayMACOSPattern::DeadOnly => chance_rate.dead_only,
        }
    }

    pub fn get_latest_chance(self, latest_chance: &LatestChance) -> LatestChance {
        match self {
            DisplayMACOSPattern::All => *latest_chance,
            DisplayMACOSPattern::GoldenOnly => LatestChance {
                latest_golden_chance: latest_chance.latest_golden_chance,
                latest_dead_chance: None,
            },
            DisplayMACOSPattern::DeadOnly => LatestChance {
                latest_golden_chance: None,
                latest_dead_chance: latest_chance.latest_dead_chance,
            },
        }
    }

    pub fn latest_chance_within_days(
        self,
        within_days: NaiveDate,
        latest_chance: &LatestChance,
    ) -> Option<(ChanceLoss, NaiveDate)> {
        let LatestChance {
            latest_golden_chance,
            latest_dead_chance,
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
                        Some((ChanceLoss::GoldenChance, latest_golden_chance))
                    } else {
                        None
                    }
                } else if latest_dead_chance >= within_days {
                    Some((ChanceLoss::DeadChance, latest_dead_chance))
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
                    Some((ChanceLoss::GoldenChance, latest_golden_chance))
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
                    Some((ChanceLoss::DeadChance, latest_dead_chance))
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

    use crate::domain::entity::close_macos_trend_analysis::{ChanceRate, LatestChance};
    use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

    #[test]
    fn display_macos_direction_test() -> Result<()> {
        let chance_rate = ChanceRate {
            all: 0.0,
            golden_only: 1.0,
            dead_only: 2.0,
        };
        let latest_chance = LatestChance {
            latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
            latest_dead_chance: NaiveDate::from_ymd_opt(2022, 12, 12),
        };
        let display_macos_pattern = DisplayMACOSPattern::All;
        let actual = display_macos_pattern.get_chance_rate(&chance_rate);
        assert_eq!(actual, 0.0);
        let actual = display_macos_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
                latest_dead_chance: NaiveDate::from_ymd_opt(2022, 12, 12),
            }
        );
        let display_macos_pattern = DisplayMACOSPattern::GoldenOnly;
        let actual = display_macos_pattern.get_chance_rate(&chance_rate);
        assert_eq!(actual, 1.0);
        let actual = display_macos_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
                latest_dead_chance: None,
            }
        );
        let display_macos_pattern = DisplayMACOSPattern::DeadOnly;
        let actual = display_macos_pattern.get_chance_rate(&chance_rate);
        assert_eq!(actual, 2.0);
        let actual = display_macos_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                latest_golden_chance: None,
                latest_dead_chance: NaiveDate::from_ymd_opt(2022, 12, 12),
            }
        );
        Ok(())
    }
}
