use crate::domain::entity::chance_loss::{ChanceLoss, CrossDirectionChangePair};
use crate::domain::entity::cross::{Cross, CrossDirection, CrossDirectionType};
use crate::domain::entity::stock::Stock;
use chrono::NaiveDate;
use itertools::Itertools;
use std::ops::Sub;

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
pub struct VecTrendAnalysis<const N: usize>(pub Vec<TrendAnalysis<N>>);

impl<'a, const N: usize> From<StockCrossPair<'a>> for VecTrendAnalysis<N> {
    fn from(value: StockCrossPair<'a>) -> Self {
        let StockCrossPair { stocks, crosses } = value;
        let trends = crosses
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
        VecTrendAnalysis::<N>(trends)
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross::VecCross;
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::stock::VecStock;
    use crate::domain::entity::trend_analysis::{StockCrossPair, VecTrendAnalysis};
    use crate::infrastructure::stock_repository::data_format::csv::VecCSVFormat;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn vec_trend_analysis_test() {
        let vec_csv_format = VecCSVFormat::<true>::from(CSV_8473);
        let VecStock(stocks) = VecStock::from(vec_csv_format);
        let VecSMA(smas_5) = VecSMA::<5>::from(stocks.as_slice());
        let VecSMA(smas_25) = VecSMA::<25>::from(stocks.as_slice());
        let sma_list_pair = SMAListPair {
            smas_n: smas_5.as_slice(),
            smas_o: smas_25.as_slice(),
        };
        let VecCross(crosses) = VecCross::from(sma_list_pair);
        let stock_cross_pair = StockCrossPair {
            stocks: stocks.as_slice(),
            crosses: crosses.as_slice(),
        };
        // 3日後トレンドを取得
        let VecTrendAnalysis(trends) = VecTrendAnalysis::<3>::from(stock_cross_pair);
        let actual = 11;
        assert_eq!(actual, trends.len());
    }
}
