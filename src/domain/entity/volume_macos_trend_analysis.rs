use chrono::NaiveDate;
use itertools::Itertools;

use crate::domain::entity::stocks_macoses_pair::StocksMACOSESPair;
use crate::domain::models::macos::model::Pattern;
use crate::domain::models::macos::model::Pattern::Neither;

/// 出来高ベースのMovingAverageCrossoverStrategyのトレンド解析
pub struct VolumeMACOSTrendAnalysis {
    /// Crossover発生日
    pub date: NaiveDate,
    /// Crossover発生日の出来高
    pub volume_on_macos: u64,
    /// MovingAverageCrossoverStrategyPattern
    pub pattern: Pattern,
}

pub struct VecVolumeMACOSTrendAnalysis(pub Vec<VolumeMACOSTrendAnalysis>);

impl<'a> From<&StocksMACOSESPair<'a>> for VecVolumeMACOSTrendAnalysis {
    fn from(value: &StocksMACOSESPair<'a>) -> Self {
        let StocksMACOSESPair { stocks, macoses } = value;
        let vec_volume_macos_trend_analysis = macoses
            .iter()
            .filter_map(|macos| {
                // Neitherは判断材料とならないため結果から除外する
                if macos.pattern_close_volume.volume.eq(&Neither) {
                    return None;
                }
                // Crossover発生日と同じ日の株価情報を取得
                let stock = stocks.iter().find(|stock| stock.date.eq(&macos.date))?;
                Some(VolumeMACOSTrendAnalysis {
                    date: stock.date,
                    volume_on_macos: stock.volume,
                    pattern: macos.pattern_close_volume.volume,
                })
            })
            .collect_vec();
        VecVolumeMACOSTrendAnalysis(vec_volume_macos_trend_analysis)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::stocks_macoses_pair::StocksMACOSESPair;
    use crate::domain::entity::volume_macos_trend_analysis::VecVolumeMACOSTrendAnalysis;
    use crate::domain::models::macos::model::MACOSES;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    #[test]
    fn vec_volume_macos_trend_analysis_test() {
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
        let VecVolumeMACOSTrendAnalysis(vec_volume_macos_trend_analysis) =
            VecVolumeMACOSTrendAnalysis::from(&stocks_macoses_pair);
        let actual = 33;
        assert_eq!(actual, vec_volume_macos_trend_analysis.len());
    }
}
