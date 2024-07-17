use chrono::NaiveDate;
use itertools::Itertools;

use crate::domain::entity::cross::CrossDirectionType;
use crate::domain::entity::cross::CrossDirectionType::Neither;
use crate::domain::entity::stocks_crosses_pair::StocksCrossesPair;

/// 出来高ベースのクロストレンド
pub struct VolumeCrossTrendAnalysis {
    /// 判定日
    pub date: NaiveDate,
    /// 判定日の出来高
    pub value_on_cross: u64,
    /// クロス方向
    pub r#type: CrossDirectionType,
}

pub struct VecVolumeCrossTrendAnalysis(pub Vec<VolumeCrossTrendAnalysis>);

impl<'a> From<&StocksCrossesPair<'a>> for VecVolumeCrossTrendAnalysis {
    fn from(value: &StocksCrossesPair<'a>) -> Self {
        let StocksCrossesPair { stocks, crosses } = value;
        let vec_volume_cross_trend_analysis = crosses
            .iter()
            .filter_map(|cross| {
                // Neitherは判断材料とならないため結果から除外する
                if cross.cross_direction_5_25.volume_average.eq(&Neither) {
                    return None;
                }
                // クロス発生日と同じ日の株価情報を取得
                let Some(stock) = stocks.iter().find(|stock| {
                    stock.date.eq(&cross.date)
                }) else { return None };
                Some(VolumeCrossTrendAnalysis {
                    date: stock.date,
                    value_on_cross: stock.volume,
                    r#type: cross.cross_direction_5_25.volume_average,
                })
            })
            .collect_vec();
        VecVolumeCrossTrendAnalysis(vec_volume_cross_trend_analysis)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross::VecCross;
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::stocks_crosses_pair::StocksCrossesPair;
    use crate::domain::entity::volume_cross_trend_analysis::VecVolumeCrossTrendAnalysis;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");
    #[test]
    fn vec_volume_cross_trend_analysis_test() {
        let vec_stock = Csv::from_slice::<true>(CSV_8473);
        let VecSMA(smas_5) = VecSMA::<5>::from(vec_stock.as_slice());
        let VecSMA(smas_25) = VecSMA::<25>::from(vec_stock.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(sma_list_pair);
        let stocks_crosses_pair = StocksCrossesPair {
            stocks: vec_stock.as_slice(),
            crosses: crosses.as_slice(),
        };
        let VecVolumeCrossTrendAnalysis(vec_volume_cross_trend_analysis) =
            VecVolumeCrossTrendAnalysis::from(&stocks_crosses_pair);
        let actual = 33;
        assert_eq!(actual, vec_volume_cross_trend_analysis.len());
    }
}
