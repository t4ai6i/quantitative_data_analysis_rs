use crate::domain::entity::candle_stick::VecCandleStick;
use crate::domain::entity::chance_loss::ChanceLoss;
use crate::domain::entity::company::Company;
use crate::domain::entity::cross::VecCross;
use crate::domain::entity::cross_trend_analysis::{
    ChanceRate, LatestChance, VecCrossTrendAnalysis,
};
use crate::domain::entity::engulfing_candlestick_pattern::VecEngulfingCandlestickPattern;
use crate::domain::entity::sma::VecSMA;
use crate::domain::entity::stock::VecStock;
use anyhow::Result;
use chrono::NaiveDate;
use strum::Display;

pub mod chart;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Display)]
pub enum DisplayCrossPattern {
    #[default]
    All,
    GoldenOnly,
    DeadOnly,
}

impl DisplayCrossPattern {
    pub fn get_chance_rate(self, chance_rate: &ChanceRate) -> f64 {
        match self {
            DisplayCrossPattern::All => chance_rate.all,
            DisplayCrossPattern::GoldenOnly => chance_rate.golden_only,
            DisplayCrossPattern::DeadOnly => chance_rate.dead_only,
        }
    }

    pub fn get_latest_chance(self, latest_chance: &LatestChance) -> LatestChance {
        match self {
            DisplayCrossPattern::All => *latest_chance,
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
            DisplayCrossPattern::All => {
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
            DisplayCrossPattern::GoldenOnly => {
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
            DisplayCrossPattern::DeadOnly => {
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

pub struct TrendAnalysisOutput<const N: usize, const M: usize> {
    company: Company,
    vec_stock: VecStock,
    vec_sma_5: VecSMA<5>,
    vec_sma_25: VecSMA<25>,
    vec_cross: VecCross,
    vec_trend: VecCrossTrendAnalysis<N>,
    vec_engulfing_candlestick_pattern: VecEngulfingCandlestickPattern,
    vec_candle_stick: VecCandleStick<M>,
    display_cross_pattern: DisplayCrossPattern,
}

impl<const N: usize, const M: usize> TrendAnalysisOutput<N, M> {
    pub fn new(
        company: Company,
        vec_stock: VecStock,
        vec_sma_5: VecSMA<5>,
        vec_sma_25: VecSMA<25>,
        vec_cross: VecCross,
        vec_trend: VecCrossTrendAnalysis<N>,
        vec_engulfing_candlestick_pattern: VecEngulfingCandlestickPattern,
        vec_candle_stick: VecCandleStick<M>,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Self {
        Self {
            company,
            vec_stock,
            vec_sma_5,
            vec_sma_25,
            vec_cross,
            vec_trend,
            vec_engulfing_candlestick_pattern,
            vec_candle_stick,
            display_cross_pattern,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TrendAnalysisResponse {
    Chart {
        company: Company,
        body: String,
        display_cross_pattern: DisplayCrossPattern,
        chance_rate: ChanceRate,
        latest_chance: LatestChance,
        vec_engulfing_candlestick_pattern: VecEngulfingCandlestickPattern,
    },
}

pub trait TrendAnalysisPresenter {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: TrendAnalysisOutput<N, M>,
    ) -> Result<TrendAnalysisResponse>;
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross_trend_analysis::{ChanceRate, LatestChance};
    use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
    use anyhow::Result;
    use chrono::NaiveDate;

    #[test]
    fn display_cross_direction_test() -> Result<()> {
        let chance_rate = ChanceRate {
            all: 0.0,
            golden_only: 1.0,
            dead_only: 2.0,
        };
        let latest_chance = LatestChance {
            latest_golden_chance: NaiveDate::from_ymd_opt(2023, 12, 12),
            latest_dead_chance: NaiveDate::from_ymd_opt(2022, 12, 12),
        };
        let display_cross_pattern = DisplayCrossPattern::All;
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
