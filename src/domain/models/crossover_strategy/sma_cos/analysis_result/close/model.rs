use crate::domain::models::crossover_strategy::analysis_pattern::model::{
    AnalysisPattern, CrossoverPatternRateOfChangePair,
};
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern::Neither;
use crate::domain::models::crossover_strategy::latest_chance::model::LatestChance;
use crate::domain::models::crossover_strategy::rate_of_chance::model::RateOfChance;
use crate::domain::models::stocks_sma_coses_pair::model::StocksSmaCosesPair;
use crate::shared::float::percentage;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use std::ops::{Mul, Sub};

/// 終値ベースの解析結果
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct AnalysisResult {
    /// Crossover発生日
    pub date_of_event: NaiveDate,
    /// Crossover発生日の終値
    pub close: f64,
    /// N日後の終値
    pub close_after_n_days: f64,
    /// CrossoverPattern
    pub pattern: CrossoverPattern,
    /// 増減率
    pub rate_of_change: f64,
    /// 分析パターン
    pub analysis_pattern: AnalysisPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct AnalysisResults<const N: usize>(Vec<AnalysisResult>);

impl<'a, const N: usize> From<&StocksSmaCosesPair<'a>> for AnalysisResults<N> {
    fn from(value: &StocksSmaCosesPair<'a>) -> Self {
        let StocksSmaCosesPair { stocks, sma_coses } = value;
        let vec_analysis_result: Vec<AnalysisResult> = sma_coses
            .par_iter()
            .filter_map(|sma_cos| {
                if sma_cos.crossover_pattern_close.0.eq(&Neither) {
                    return None;
                }
                // Crossoverが発生した日を特定
                let (stock, sma_cos) = stocks.par_iter().find_map_first(|stock| {
                    if stock.date.eq(&sma_cos.date) {
                        Some((stock, sma_cos))
                    } else {
                        None
                    }
                })?;
                // n日後のStockを取得。stocksは営業日で並んでいる。
                let (stock, sma_cos, stock_after_n_days) = stocks
                    .par_iter()
                    .enumerate()
                    .find_first(|(_, stock)| stock.date.eq(&sma_cos.date))
                    .and_then(|(index, _)| {
                        let stock_after_n_days = stocks.get(index + N)?;
                        Some((stock, sma_cos, stock_after_n_days))
                    })?;
                // 増減率を取得
                let rate_of_change = stock_after_n_days.close.sub(stock.close) / stock.close;
                let rate_of_change = rate_of_change.mul(100.0);
                let pattern_rate_of_change_pair = CrossoverPatternRateOfChangePair {
                    crossover_pattern: sma_cos.crossover_pattern_close.0,
                    rate_of_change,
                };
                let analysis_pattern = AnalysisPattern::from(pattern_rate_of_change_pair);
                Some(AnalysisResult {
                    date_of_event: sma_cos.date,
                    close: stock.close,
                    close_after_n_days: stock_after_n_days.close,
                    pattern: sma_cos.crossover_pattern_close.0,
                    rate_of_change,
                    analysis_pattern,
                })
            })
            .collect();
        AnalysisResults::<N>(vec_analysis_result)
    }
}

impl<const N: usize> AnalysisResults<N> {
    pub fn rate_of_chance(&self) -> RateOfChance {
        let count_golden_chance = self
            .par_iter()
            .filter(|analysis_result| {
                matches!(
                    analysis_result.analysis_pattern,
                    AnalysisPattern::GoldenChance
                )
            })
            .count();
        let count_golden_loss = self
            .par_iter()
            .filter(|analysis_result| {
                matches!(
                    analysis_result.analysis_pattern,
                    AnalysisPattern::GoldenLoss
                )
            })
            .count();
        let golden = percentage(count_golden_chance, count_golden_chance + count_golden_loss);

        let count_dead_chance = self
            .par_iter()
            .filter(|analysis_result| {
                matches!(
                    analysis_result.analysis_pattern,
                    AnalysisPattern::DeadChance
                )
            })
            .count();
        let count_dead_loss = self
            .par_iter()
            .filter(|analysis_result| {
                matches!(analysis_result.analysis_pattern, AnalysisPattern::DeadLoss)
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
            .find_last(|analysis_result| analysis_result.analysis_pattern.eq(target_pattern))
            .map(|analysis_result| analysis_result.date_of_event)
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
    use crate::domain::models::crossover_strategy::sma_cos::analysis_result::close::model::AnalysisResults;
    use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_sma_coses_pair::model::StocksSmaCosesPair;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;
    use anyhow::Result;
    use bytes::Bytes;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;

    const CSV: &[u8] = include_bytes!("../../../../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn sma_cos_analysis_close_test() -> Result<()> {
        let dsv = Dsv::<csv::Structure>::new(true, Bytes::from(CSV));
        let query = queries::get_stocks::Query {
            ..Default::default()
        };
        let stocks = dsv.get_stocks(&query).await?;
        let smas_5 = SMAs::<5>::from(stocks.as_slice());
        let smas_25 = SMAs::<25>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let sma_coses = SmaCoses::from(sma_list_pair);
        let stocks_sma_coses_pair = StocksSmaCosesPair {
            stocks: stocks.as_slice(),
            sma_coses: sma_coses.as_slice(),
        };
        // 3日後トレンドを取得
        let analysis_results = AnalysisResults::<3>::from(&stocks_sma_coses_pair);
        let actual = 11;
        assert_eq!(actual, analysis_results.len());

        let rate_of_chance = analysis_results.rate_of_chance();
        let actual = 45.45454545454545;
        assert_eq!(actual, rate_of_chance.whole);
        let actual = 66.66666666666666;
        assert_eq!(actual, rate_of_chance.golden);
        let actual = 20.0;
        assert_eq!(actual, rate_of_chance.dead);

        let latest_chance = analysis_results.latest_chance();
        let actual = NaiveDate::from_ymd_opt(2023, 8, 30);
        assert_eq!(actual, latest_chance.golden_cross);
        let actual = NaiveDate::from_ymd_opt(2023, 3, 14);
        assert_eq!(actual, latest_chance.dead_cross);
        Ok(())
    }
}
