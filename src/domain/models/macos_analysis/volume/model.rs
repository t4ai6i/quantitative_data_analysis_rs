use crate::domain::models::macos::model::Pattern;
use crate::domain::models::macos::model::Pattern::Neither;
use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use rayon::prelude::*;

/// 出来高ベースのMovingAverageCrossoverStrategyのトレンド解析
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct MACOSAnalysisVolume {
    /// Crossover発生日
    pub date_of_event: NaiveDate,
    /// Crossover発生日の出来高
    pub volume: u64,
    /// MovingAverageCrossoverStrategyPattern
    pub pattern: Pattern,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct MACOSAnalysisVolumes(Vec<MACOSAnalysisVolume>);

impl<'a> From<&StocksMACOSESPair<'a>> for MACOSAnalysisVolumes {
    fn from(value: &StocksMACOSESPair<'a>) -> Self {
        let StocksMACOSESPair { stocks, macoses } = value;
        let vec_macos_analysis_volume = macoses
            .par_iter()
            .filter_map(|macos| {
                // Neitherは判断材料とならないため結果から除外する
                if macos.pattern_close_volume.volume.eq(&Neither) {
                    return None;
                }
                // Crossover発生日と同じ日の株価情報を取得
                let stock = stocks
                    .par_iter()
                    .find_first(|stock| stock.date.eq(&macos.date))?;
                Some(MACOSAnalysisVolume {
                    date_of_event: stock.date,
                    volume: stock.volume,
                    pattern: macos.pattern_close_volume.volume,
                })
            })
            .collect::<Vec<MACOSAnalysisVolume>>();
        MACOSAnalysisVolumes(vec_macos_analysis_volume)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::macos::model::MACOSes;
    use crate::domain::models::macos_analysis::volume::model::MACOSAnalysisVolumes;
    use crate::domain::models::sma::model::{SMAListPair, SMAs};
    use crate::domain::models::stocks_macoses_pair::model::StocksMACOSESPair;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::stock::data_format::csv::Csv;
    use rayon::prelude::*;

    const CSV_8473: &[u8] = include_bytes!("../../../../../assets/8473.T.csv");

    #[test]
    fn macos_analysis_volume_test() {
        let successes: Vec<_> = Csv::from_slice::<true>(CSV_8473)
            .into_par_iter()
            .map(|s| s.unwrap())
            .collect();
        let vec_stock: Vec<_> = Csv::from_deserialize(successes)
            .into_par_iter()
            .map(|s| s.unwrap())
            .collect();
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
        let macos_analysis_volumes = MACOSAnalysisVolumes::from(&stocks_macoses_pair);
        let actual = 33;
        assert_eq!(actual, macos_analysis_volumes.len());
    }
}
