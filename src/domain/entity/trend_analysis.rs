use crate::domain::entity::chance_loss::{ChanceLoss, CrossDirectionChangePair};
use crate::domain::entity::cross::{Cross, CrossDirection, CrossDirectionType};
use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use std::ops::{Mul, Sub};
use CrossDirectionType::{Dead, Golden};

pub struct StockCrossPair<'a> {
    pub stocks: &'a [Stock],
    pub crosses: &'a [Cross],
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct TrendAnalysis<const N: usize> {
    pub cross_date: NaiveDate,
    pub close_on_cross: f64,
    pub close_after_n_days: f64,
    pub cross_direction_5_25: CrossDirection<5, 25>,
    pub change: f64,
    pub chance_loss: ChanceLoss,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct ChanceRate {
    pub all: f64,
    pub golden_only: f64,
    pub dead_only: f64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct LatestChance {
    pub latest_golden_chance: Option<NaiveDate>,
    pub latest_dead_chance: Option<NaiveDate>,
}

impl From<LatestChance> for CrossDirectionType {
    fn from(value: LatestChance) -> Self {
        let LatestChance {
            latest_golden_chance,
            latest_dead_chance,
        } = value;
        match (latest_golden_chance, latest_dead_chance) {
            (Some(golden), Some(dead)) => {
                if golden >= dead {
                    Golden
                } else {
                    Dead
                }
            }
            (Some(_), None) => Golden,
            (None, Some(_)) => Dead,
            _ => CrossDirectionType::None,
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
pub struct VecTrendAnalysis<const N: usize> {
    pub vec_trend_analysis: Vec<TrendAnalysis<N>>,
    pub chance_rate: ChanceRate,
    pub latest_chance: LatestChance,
}

impl<'a, const N: usize> From<StockCrossPair<'a>> for VecTrendAnalysis<N> {
    fn from(value: StockCrossPair<'a>) -> Self {
        let StockCrossPair { stocks, crosses } = value;
        let vec_trend_analysis = crosses
            .iter()
            .filter(|cross| cross.cross_direction_5_25.0.ne(&CrossDirectionType::None))
            .filter_map(|cross| {
                // Crossの発生した日を特定
                stocks.iter().find_map(|stock| {
                    if stock.date.eq(&cross.date) {
                        Some((stock, cross))
                    } else {
                        None
                    }
                })
            })
            .filter_map(|(stock, cross)| {
                // n日後のStockを取得。ただし営業日で並んでいる。
                stocks
                    .iter()
                    .find_position(|stock| stock.date.eq(&cross.date))
                    .and_then(|(index, _)| stocks.get(index + N))
                    .map(|stock_after_n_days| (stock, cross, stock_after_n_days))
            })
            .map(|(stock, cross, stock_after_n_days)| {
                // 増減率を取得
                let change = stock_after_n_days.close.sub(stock.close) / stock.close;
                let change = change.mul(100.0);
                let cross_direction_chance_pair = CrossDirectionChangePair {
                    cross_direction: cross.cross_direction_5_25.0,
                    change,
                };
                let chance_loss = ChanceLoss::from(cross_direction_chance_pair);
                TrendAnalysis {
                    cross_date: cross.date,
                    close_on_cross: stock.close,
                    close_after_n_days: stock_after_n_days.close,
                    cross_direction_5_25: cross.cross_direction_5_25,
                    change,
                    chance_loss,
                }
            })
            .collect_vec();
        let golden_chance_count = vec_trend_analysis
            .iter()
            .filter(|trend_analysis| matches!(trend_analysis.chance_loss, ChanceLoss::GoldenChance))
            .count();
        let golden_loss_count = vec_trend_analysis
            .iter()
            .filter(|trend_analysis| matches!(trend_analysis.chance_loss, ChanceLoss::GoldenLoss))
            .count();
        let golden_only = (golden_chance_count as f64
            / (golden_chance_count + golden_loss_count) as f64)
            .mul(100.0);
        let dead_chance_count = vec_trend_analysis
            .iter()
            .filter(|trend_analysis| matches!(trend_analysis.chance_loss, ChanceLoss::DeadChance))
            .count();
        let dead_loss_count = vec_trend_analysis
            .iter()
            .filter(|trend_analysis| matches!(trend_analysis.chance_loss, ChanceLoss::DeadLoss))
            .count();
        let dead_only =
            (dead_chance_count as f64 / (dead_chance_count + dead_loss_count) as f64).mul(100.0);
        let chance_count = vec_trend_analysis
            .iter()
            .filter(|trend_analysis| match trend_analysis.chance_loss {
                ChanceLoss::None => false,
                ChanceLoss::GoldenChance => true,
                ChanceLoss::DeadChance => true,
                ChanceLoss::GoldenLoss => false,
                ChanceLoss::DeadLoss => false,
            })
            .count();
        let all = (chance_count as f64 / vec_trend_analysis.len() as f64).mul(100.0);
        let chance_rate = ChanceRate {
            all,
            golden_only,
            dead_only,
        };
        let latest_golden_chance = vec_trend_analysis
            .iter()
            .rev()
            .find(|trend_analysis| trend_analysis.chance_loss.eq(&ChanceLoss::GoldenChance))
            .map(|trend_analysis| trend_analysis.cross_date);
        let latest_dead_chance = vec_trend_analysis
            .iter()
            .rev()
            .find(|trend_analysis| trend_analysis.chance_loss.eq(&ChanceLoss::DeadChance))
            .map(|trend_analysis| trend_analysis.cross_date);
        let latest_chance = LatestChance {
            latest_golden_chance,
            latest_dead_chance,
        };
        VecTrendAnalysis::<N> {
            vec_trend_analysis,
            chance_rate,
            latest_chance,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross::VecCross;
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::trend_analysis::{StockCrossPair, VecTrendAnalysis};
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use chrono::NaiveDate;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn vec_trend_analysis_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
        let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(sma_list_pair);
        let stock_cross_pair = StockCrossPair {
            stocks: vec_stock.as_slice(),
            crosses: crosses.as_slice(),
        };
        // 3日後トレンドを取得
        let VecTrendAnalysis {
            vec_trend_analysis,
            chance_rate,
            latest_chance,
        } = VecTrendAnalysis::<3>::from(stock_cross_pair);
        let actual = 11;
        assert_eq!(actual, vec_trend_analysis.len());
        let actual = 45.45454545454545;
        assert_eq!(actual, chance_rate.all);
        let actual = 66.66666666666666;
        assert_eq!(actual, chance_rate.golden_only);
        let actual = 20.0;
        assert_eq!(actual, chance_rate.dead_only);
        let actual = NaiveDate::from_ymd_opt(2023, 8, 30);
        assert_eq!(actual, latest_chance.latest_golden_chance);
        let actual = NaiveDate::from_ymd_opt(2023, 3, 14);
        assert_eq!(actual, latest_chance.latest_dead_chance);
    }
}
