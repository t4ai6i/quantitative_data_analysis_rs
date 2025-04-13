use crate::domain::models::macos::model::Pattern::{DeadCross, GoldenCross, Neither};
use crate::domain::models::macos::model::{AnalysisPattern, Pattern, PatternRateOfChangePair};
use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
use crate::utils::float::percentage;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;
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

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct MACOSAnalysisCloses<const N: usize>(Vec<MACOSAnalysisClose>);

impl<'a, const N: usize> From<&StocksMACOSESPair<'a>> for MACOSAnalysisCloses<N> {
    fn from(value: &StocksMACOSESPair<'a>) -> Self {
        let StocksMACOSESPair { stocks, macoses } = value;
        let vec_macos_analysis_close: Vec<MACOSAnalysisClose> = macoses
            .par_iter()
            .filter_map(|macos| {
                if macos.pattern_close_volume.close.eq(&Neither) {
                    return None;
                }
                // Crossoverが発生した日を特定
                let (stock, macos) = stocks.par_iter().find_map_first(|stock| {
                    if stock.date.eq(&macos.date) {
                        Some((stock, macos))
                    } else {
                        None
                    }
                })?;
                // n日後のStockを取得。ただし営業日で並んでいる。
                let (stock, macos, stock_after_n_days) = stocks
                    .par_iter()
                    .enumerate()
                    .find_first(|(_, stock)| stock.date.eq(&macos.date))
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
            .collect();
        MACOSAnalysisCloses::<N>(vec_macos_analysis_close)
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct RateOfChance {
    pub whole: f64,
    pub golden: f64,
    pub dead: f64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct LatestChance {
    pub golden_cross: Option<NaiveDate>,
    pub dead_cross: Option<NaiveDate>,
}

impl From<LatestChance> for Pattern {
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

impl<const N: usize> MACOSAnalysisCloses<N> {
    pub fn rate_of_chance(&self) -> RateOfChance {
        let count_golden_chance = self
            .par_iter()
            .filter(|macos_analysis_close| {
                matches!(
                    macos_analysis_close.analysis_pattern,
                    AnalysisPattern::GoldenChance
                )
            })
            .count();
        let count_golden_loss = self
            .par_iter()
            .filter(|macos_analysis_close| {
                matches!(
                    macos_analysis_close.analysis_pattern,
                    AnalysisPattern::GoldenLoss
                )
            })
            .count();
        let golden = percentage(count_golden_chance, count_golden_chance + count_golden_loss);

        let count_dead_chance = self
            .par_iter()
            .filter(|macos_analysis_close| {
                matches!(
                    macos_analysis_close.analysis_pattern,
                    AnalysisPattern::DeadChance
                )
            })
            .count();
        let count_dead_loss = self
            .par_iter()
            .filter(|macos_analysis_close| {
                matches!(
                    macos_analysis_close.analysis_pattern,
                    AnalysisPattern::DeadLoss
                )
            })
            .count();
        let dead = percentage(count_dead_chance, count_dead_chance + count_dead_loss);

        let whole = percentage(count_golden_chance + count_dead_chance, self.len());
        RateOfChance {
            whole,
            golden,
            dead,
        }
    }

    fn find_latest_event(&self, target_pattern: &AnalysisPattern) -> Option<NaiveDate> {
        self.par_iter()
            .rev()
            .find_first(|macos_analysis_close| {
                macos_analysis_close.analysis_pattern.eq(target_pattern)
            })
            .map(|macos_analysis_close| macos_analysis_close.date_of_event)
    }

    pub fn latest_chance(&self) -> LatestChance {
        let golden_cross = self.find_latest_event(&AnalysisPattern::GoldenChance);
        let dead_cross = self.find_latest_event(&AnalysisPattern::DeadChance);
        LatestChance {
            golden_cross,
            dead_cross,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::macos::model::MACOSes;
    use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::stock::data_format::csv::Csv;
    use chrono::NaiveDate;

    const CSV_8473: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");

    #[test]
    fn macos_analysis_close_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let smas_5 = SMAs::<5>::from(vec_stock.as_slice());
        let smas_25 = SMAs::<25>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let macoses = MACOSes::from(sma_list_pair);
        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: vec_stock.as_slice(),
            macoses: macoses.as_slice(),
        };
        // 3日後トレンドを取得
        let macos_analysis_closes = MACOSAnalysisCloses::<3>::from(&stocks_macoses_pair);
        let actual = 11;
        assert_eq!(actual, macos_analysis_closes.len());

        let rate_of_chance = macos_analysis_closes.rate_of_chance();
        let actual = 45.45454545454545;
        assert_eq!(actual, rate_of_chance.whole);
        let actual = 66.66666666666666;
        assert_eq!(actual, rate_of_chance.golden);
        let actual = 20.0;
        assert_eq!(actual, rate_of_chance.dead);

        let latest_chance = macos_analysis_closes.latest_chance();
        let actual = NaiveDate::from_ymd_opt(2023, 8, 30);
        assert_eq!(actual, latest_chance.golden_cross);
        let actual = NaiveDate::from_ymd_opt(2023, 3, 14);
        assert_eq!(actual, latest_chance.dead_cross);
    }
}
