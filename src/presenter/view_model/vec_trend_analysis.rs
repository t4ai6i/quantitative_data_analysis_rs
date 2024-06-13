use crate::domain::entity::chance_loss::ChanceLoss;
use crate::domain::entity::cross::CrossDirectionType;
use crate::domain::entity::cross_trend_analysis::VecCrossTrendAnalysis;
use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
use itertools::Itertools;

pub trait VecTrendAnalysisExt {
    fn table_chart_rows(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>>;
    fn table_chart_summary(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>>;
}

impl<const N: usize> VecTrendAnalysisExt for VecCrossTrendAnalysis<N> {
    fn table_chart_rows(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>> {
        self.vec_trend_analysis
            .iter()
            .filter(|trend_analysis| match pattern {
                DisplayCrossPattern::All => true,
                DisplayCrossPattern::GoldenOnly => trend_analysis
                    .cross_direction_5_25
                    .0
                    .eq(&CrossDirectionType::Golden),
                DisplayCrossPattern::DeadOnly => trend_analysis
                    .cross_direction_5_25
                    .0
                    .eq(&CrossDirectionType::Dead),
            })
            .map(|trend_analysis| {
                let chance_loss = match trend_analysis.chance_loss {
                    ChanceLoss::None => "❔".to_string(),
                    ChanceLoss::GoldenChance => "✅".to_string(),
                    ChanceLoss::DeadChance => "✅".to_string(),
                    ChanceLoss::GoldenLoss => "❌".to_string(),
                    ChanceLoss::DeadLoss => "❌".to_string(),
                };
                let cross_date = trend_analysis.cross_date.format("%Y/%m/%d").to_string();
                let cross_direction_5_25 = trend_analysis.cross_direction_5_25.0.to_string();
                let change = format!("{:+.3}%", trend_analysis.change);
                let close_on_cross = trend_analysis.close_on_cross.to_string();
                let close_after_n_days = trend_analysis.close_after_n_days.to_string();
                vec![
                    cross_date,
                    chance_loss,
                    cross_direction_5_25,
                    change,
                    close_on_cross,
                    close_after_n_days,
                ]
            })
            .collect_vec()
    }

    fn table_chart_summary(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>> {
        let chance_rate = match pattern {
            DisplayCrossPattern::All => {
                format!("{:.0}%", self.chance_rate.all)
            }
            DisplayCrossPattern::GoldenOnly => {
                format!("{:.0}%", self.chance_rate.golden_only)
            }
            DisplayCrossPattern::DeadOnly => {
                format!("{:.0}%", self.chance_rate.dead_only)
            }
        };
        vec![vec![
            "".to_string(),
            chance_rate,
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "".to_string(),
        ]]
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::cross::VecCross;
    use crate::domain::entity::cross_trend_analysis::{StockCrossPair, VecCrossTrendAnalysis};
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;
    use crate::presenter::view_model::vec_trend_analysis::VecTrendAnalysisExt;
    use anyhow::Result;

    const CSV_8473: &[u8] = include_bytes!("../../../assets/8473.T.csv");

    #[test]
    fn table_chart_summary_test() -> Result<()> {
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
        let vec_trend = VecCrossTrendAnalysis::<5>::from(stock_cross_pair);
        let summary = vec_trend.table_chart_summary(&DisplayCrossPattern::All);
        assert_eq!(
            summary,
            vec![vec![
                "".to_string(),
                "27%".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]]
        );
        let summary = vec_trend.table_chart_summary(&DisplayCrossPattern::GoldenOnly);
        assert_eq!(
            summary,
            vec![vec![
                "".to_string(),
                "33%".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]]
        );
        let summary = vec_trend.table_chart_summary(&DisplayCrossPattern::DeadOnly);
        assert_eq!(
            summary,
            vec![vec![
                "".to_string(),
                "20%".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ]]
        );
        Ok(())
    }
}
