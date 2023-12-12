use crate::domain::entity::company::Company;
use crate::domain::entity::cross::VecCross;
use crate::domain::entity::sma::VecSMA;
use crate::domain::entity::stock::VecStock;
use crate::domain::entity::trend_analysis::{ChanceRate, LatestChance, VecTrendAnalysis};
use anyhow::Result;

pub mod chart;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum DisplayCrossPattern {
    #[default]
    Both,
    GoldenOnly,
    DeadOnly,
}

impl DisplayCrossPattern {
    pub fn get_chance_rate(self, chance_rate: &ChanceRate) -> f64 {
        match self {
            DisplayCrossPattern::Both => chance_rate.total,
            DisplayCrossPattern::GoldenOnly => chance_rate.golden_only,
            DisplayCrossPattern::DeadOnly => chance_rate.dead_only,
        }
    }

    pub fn get_latest_chance(self, latest_chance: &LatestChance) -> LatestChance {
        match self {
            DisplayCrossPattern::Both => *latest_chance,
            DisplayCrossPattern::GoldenOnly => LatestChance {
                latest_golden_chance: latest_chance.latest_golden_chance,
                latest_dead_chance: None,
            },
            DisplayCrossPattern::DeadOnly => LatestChance {
                latest_golden_chance: None,
                latest_dead_chance: latest_chance.latest_dead_chance,
            },
        }
    }
}

pub struct TrendAnalysisOutput<const N: usize> {
    company: Company,
    vec_stock: VecStock,
    vec_sma_5: VecSMA<5>,
    vec_sma_25: VecSMA<25>,
    vec_cross: VecCross,
    vec_trend: VecTrendAnalysis<N>,
    display_cross_pattern: DisplayCrossPattern,
}

impl<const N: usize> TrendAnalysisOutput<N> {
    pub fn new(
        company: Company,
        vec_stock: VecStock,
        vec_sma_5: VecSMA<5>,
        vec_sma_25: VecSMA<25>,
        vec_cross: VecCross,
        vec_trend: VecTrendAnalysis<N>,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Self {
        Self {
            company,
            vec_stock,
            vec_sma_5,
            vec_sma_25,
            vec_cross,
            vec_trend,
            display_cross_pattern,
        }
    }
}

pub enum TrendAnalysisResponse {
    Chart {
        company: Company,
        body: String,
        display_cross_pattern: DisplayCrossPattern,
        chance_rate: ChanceRate,
        latest_chance: LatestChance,
    },
}

pub trait TrendAnalysisPresenter {
    fn handle<const N: usize>(
        &self,
        output: TrendAnalysisOutput<N>,
    ) -> Result<TrendAnalysisResponse>;
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::trend_analysis::{ChanceRate, LatestChance};
    use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
    use anyhow::Result;
    use chrono::NaiveDate;

    #[test]
    fn display_cross_direction_test() -> Result<()> {
        let chance_rate = ChanceRate {
            total: 0.0,
            golden_only: 1.0,
            dead_only: 2.0,
        };
        let latest_chance = LatestChance {
            latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
            latest_dead_chance: NaiveDate::from_ymd_opt(2022, 12, 12),
        };
        let display_cross_pattern = DisplayCrossPattern::Both;
        let actual = display_cross_pattern.get_chance_rate(&chance_rate);
        assert_eq!(actual, 0.0);
        let actual = display_cross_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
                latest_dead_chance: NaiveDate::from_ymd_opt(2022, 12, 12),
            }
        );
        let display_cross_pattern = DisplayCrossPattern::GoldenOnly;
        let actual = display_cross_pattern.get_chance_rate(&chance_rate);
        assert_eq!(actual, 1.0);
        let actual = display_cross_pattern.get_latest_chance(&latest_chance);
        assert_eq!(
            actual,
            LatestChance {
                latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
                latest_dead_chance: None,
            }
        );
        let display_cross_pattern = DisplayCrossPattern::DeadOnly;
        let actual = display_cross_pattern.get_chance_rate(&chance_rate);
        assert_eq!(actual, 2.0);
        let actual = display_cross_pattern.get_latest_chance(&latest_chance);
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
