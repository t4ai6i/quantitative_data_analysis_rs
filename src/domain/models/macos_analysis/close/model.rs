use crate::domain::entity::stocks_macoses_pair::StocksMACOSESPair;
use crate::domain::models::macos::model::Pattern::{DeadCross, GoldenCross, Neither};
use crate::domain::models::macos::model::{AnalysisPattern, Pattern, PatternRateOfChangePair};
use chrono::NaiveDate;
use itertools::Itertools;
use std::ops::{Mul, Sub};

/// 終値ベースのMovingAverageCrossoverStrategyの解析
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOSAnalysisClose {
    /// Crossover発生日
    pub date_of_event: NaiveDate,
    /// Crossover発生日の終値
    pub close: f64,
    /// N日後の終値
    pub close_after_n_days: f64,
    /// MovingAverageCrossoverStrategyPattern
    pub pattern: Pattern,
    /// 増減率
    pub rate_of_change: f64,
    /// 分析パターン
    pub analysis_pattern: AnalysisPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RateOfChance {
    pub all: f64,
    pub golden_only: f64,
    pub dead_only: f64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct LatestChance {
    pub latest_golden_chance: Option<NaiveDate>,
    pub latest_dead_chance: Option<NaiveDate>,
}

impl From<LatestChance> for Pattern {
    fn from(value: LatestChance) -> Self {
        let LatestChance {
            latest_golden_chance,
            latest_dead_chance,
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
            latest_golden_chance,
            latest_dead_chance,
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOSAnalysisCloses<const N: usize> {
    pub vec_macos_analysis_close: Vec<MACOSAnalysisClose>,
    pub rate_of_chance: RateOfChance,
    pub latest_chance: LatestChance,
}

impl<'a, const N: usize> From<&StocksMACOSESPair<'a>> for MACOSAnalysisCloses<N> {
    fn from(value: &StocksMACOSESPair<'a>) -> Self {
        let StocksMACOSESPair { stocks, macoses } = value;
        let vec_macos_analysis_close = macoses
            .iter()
            .filter_map(|macos| {
                if macos.pattern_close_volume.close.eq(&Neither) {
                    return None;
                }
                // Crossoverが発生した日を特定
                let (stock, macos) = stocks.iter().find_map(|stock| {
                    if stock.date.eq(&macos.date) {
                        Some((stock, macos))
                    } else {
                        None
                    }
                })?;
                // n日後のStockを取得。ただし営業日で並んでいる。
                let (stock, macos, stock_after_n_days) = stocks
                    .iter()
                    .find_position(|stock| stock.date.eq(&macos.date))
                    .and_then(|(index, _)| {
                        let stock_after_n_days = stocks.get(index + N)?;
                        Some((stock, macos, stock_after_n_days))
                    })?;
                // 増減率を取得
                let rate_of_change = stock_after_n_days.close.sub(stock.close) / stock.close;
                let rate_of_change = rate_of_change.mul(100.0);
                let pattern_rate_of_change_pair = PatternRateOfChangePair {
                    pattern: macos.pattern_close_volume.close,
                    rate_of_change,
                };
                let analysis_pattern = AnalysisPattern::from(pattern_rate_of_change_pair);
                Some(MACOSAnalysisClose {
                    date_of_event: macos.date,
                    close: stock.close,
                    close_after_n_days: stock_after_n_days.close,
                    pattern: macos.pattern_close_volume.close,
                    rate_of_change,
                    analysis_pattern,
                })
            })
            .collect_vec();
        let golden_chance_count = vec_macos_analysis_close
            .iter()
            .filter(|trend_analysis| {
                matches!(
                    trend_analysis.analysis_pattern,
                    AnalysisPattern::GoldenChance
                )
            })
            .count();
        let golden_loss_count = vec_macos_analysis_close
            .iter()
            .filter(|trend_analysis| {
                matches!(trend_analysis.analysis_pattern, AnalysisPattern::GoldenLoss)
            })
            .count();
        let golden_only = (golden_chance_count as f64
            / (golden_chance_count + golden_loss_count) as f64)
            .mul(100.0);
        let golden_only = if golden_only.is_nan() {
            0.0
        } else {
            golden_only
        };
        let dead_chance_count = vec_macos_analysis_close
            .iter()
            .filter(|trend_analysis| {
                matches!(trend_analysis.analysis_pattern, AnalysisPattern::DeadChance)
            })
            .count();
        let dead_loss_count = vec_macos_analysis_close
            .iter()
            .filter(|trend_analysis| {
                matches!(trend_analysis.analysis_pattern, AnalysisPattern::DeadLoss)
            })
            .count();
        let dead_only =
            (dead_chance_count as f64 / (dead_chance_count + dead_loss_count) as f64).mul(100.0);
        let dead_only = if dead_only.is_nan() { 0.0 } else { dead_only };
        let chance_count = vec_macos_analysis_close
            .iter()
            .filter(|trend_analysis| match trend_analysis.analysis_pattern {
                AnalysisPattern::None => false,
                AnalysisPattern::GoldenChance => true,
                AnalysisPattern::DeadChance => true,
                AnalysisPattern::GoldenLoss => false,
                AnalysisPattern::DeadLoss => false,
            })
            .count();
        let all = (chance_count as f64 / vec_macos_analysis_close.len() as f64).mul(100.0);
        let all = if all.is_nan() { 0.0 } else { all };
        let rate_of_chance = RateOfChance {
            all,
            golden_only,
            dead_only,
        };
        let latest_golden_chance = vec_macos_analysis_close
            .iter()
            .rev()
            .find(|trend_analysis| {
                trend_analysis
                    .analysis_pattern
                    .eq(&AnalysisPattern::GoldenChance)
            })
            .map(|trend_analysis| trend_analysis.date_of_event);
        let latest_dead_chance = vec_macos_analysis_close
            .iter()
            .rev()
            .find(|trend_analysis| {
                trend_analysis
                    .analysis_pattern
                    .eq(&AnalysisPattern::DeadChance)
            })
            .map(|trend_analysis| trend_analysis.date_of_event);
        let latest_chance = LatestChance {
            latest_golden_chance,
            latest_dead_chance,
        };
        MACOSAnalysisCloses::<N> {
            vec_macos_analysis_close,
            rate_of_chance,
            latest_chance,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::stocks_macoses_pair::StocksMACOSESPair;
    use crate::domain::models::macos::model::MACOSES;
    use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;

    const CSV_8473: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");

    #[test]
    fn macos_analysis_close_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
        let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let macoses = MACOSES::from(sma_list_pair);
        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: vec_stock.as_slice(),
            macoses: macoses.as_slice(),
        };
        // 3日後トレンドを取得
        let MACOSAnalysisCloses {
            vec_macos_analysis_close,
            rate_of_chance,
            latest_chance,
        } = MACOSAnalysisCloses::<3>::from(&stocks_macoses_pair);
        let actual = 11;
        assert_eq!(actual, vec_macos_analysis_close.len());
        let actual = 45.45454545454545;
        assert_eq!(actual, rate_of_chance.all);
        let actual = 66.66666666666666;
        assert_eq!(actual, rate_of_chance.golden_only);
        let actual = 20.0;
        assert_eq!(actual, rate_of_chance.dead_only);
        let actual = NaiveDate::from_ymd_opt(2023, 8, 30);
        assert_eq!(actual, latest_chance.latest_golden_chance);
        let actual = NaiveDate::from_ymd_opt(2023, 3, 14);
        assert_eq!(actual, latest_chance.latest_dead_chance);
    }
}
