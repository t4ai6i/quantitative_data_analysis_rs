use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;
use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern::Neither;
use crate::domain::models::stocks_sma_coses_pair::model::StocksSmaCosesPair;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// 出来高ベースの解析結果
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct AnalysisResult {
    /// Crossover発生日
    pub date_of_event: NaiveDate,
    /// Crossover発生日の出来高
    pub volume: u64,
    /// CrossoverPattern
    pub pattern: CrossoverPattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct AnalysisResults(Vec<AnalysisResult>);

impl<'a> From<&StocksSmaCosesPair<'a>> for AnalysisResults {
    fn from(value: &StocksSmaCosesPair<'a>) -> Self {
        let StocksSmaCosesPair { stocks, sma_coses } = value;
        let vec_sma_cos_analysis_volume = sma_coses
            .par_iter()
            .filter_map(|sma_cos| {
                // Neitherは判断材料とならないため結果から除外する
                if sma_cos.crossover_pattern_volume.0.eq(&Neither) {
                    return None;
                }
                // Crossover発生日と同じ日の株価情報を取得
                let stock = stocks
                    .par_iter()
                    .find_first(|stock| stock.date.eq(&sma_cos.date))?;
                Some(AnalysisResult {
                    date_of_event: stock.date,
                    volume: stock.volume,
                    pattern: sma_cos.crossover_pattern_volume.0,
                })
            })
            .collect::<Vec<AnalysisResult>>();
        AnalysisResults(vec_sma_cos_analysis_volume)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::crossover_strategy::sma_cos::analysis_result::volume::model::AnalysisResults;
    use crate::domain::models::crossover_strategy::sma_cos::model::SmaCoses;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_sma_coses_pair::model::StocksSmaCosesPair;
    use crate::domain::repositories::stock::queries;
    use crate::domain::repositories::stock::repository::Stock;
    use crate::infrastructure::dsv::Dsv;
    use crate::infrastructure::repositories::stock::structures::internal::csv;
    use anyhow::Result;
    use bytes::Bytes;
    use pretty_assertions::assert_eq;

    const CSV: &[u8] = include_bytes!("../../../../../../../assets/8473.T.csv");

    #[tokio::test]
    async fn sma_cos_analysis_volume_test() -> Result<()> {
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
        let sma_cos_analysis_volumes = AnalysisResults::from(&stocks_sma_coses_pair);
        let actual = 33;
        assert_eq!(actual, sma_cos_analysis_volumes.len());
        Ok(())
    }
}
