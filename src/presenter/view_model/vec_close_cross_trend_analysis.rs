use itertools::Itertools;

use crate::domain::entity::chance_loss::ChanceLoss;
use crate::domain::entity::close_cross_trend_analysis::VecCloseCrossTrendAnalysis;
use crate::presenter::display_cross_pattern::DisplayCrossPattern;

pub trait VecCloseCrossTrendAnalysisExt {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>>;
    fn table_chart_summary(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>>;
}

impl<const N: usize> VecCloseCrossTrendAnalysisExt for VecCloseCrossTrendAnalysis<N> {
    fn table_chart_header(&self) -> Vec<Vec<String>> {
        vec![vec![
            "date".to_string(),
            "chance loss".to_string(),
            "direction".to_string(),
            "per inc/dec".to_string(),
            "close".to_string(),
            format!("close after {} days", N),
        ]]
    }

    fn table_chart_rows(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>> {
        self.vec_close_cross_trend_analysis
            .iter()
            .filter(|trend_analysis| {
                pattern.is_display_by_cross_direction_type(&trend_analysis.r#type)
            })
            .map(|trend_analysis| {
                let chance_loss = match trend_analysis.chance_loss {
                    ChanceLoss::None => "❔".to_string(),
                    ChanceLoss::GoldenChance => "✅".to_string(),
                    ChanceLoss::DeadChance => "✅".to_string(),
                    ChanceLoss::GoldenLoss => "❌".to_string(),
                    ChanceLoss::DeadLoss => "❌".to_string(),
                };
                let cross_date = trend_analysis.date.format("%Y/%m/%d").to_string();
                let cross_direction_5_25 = trend_analysis.r#type.to_string();
                let change = format!("{:+.3}%", trend_analysis.rate_of_change);
                let value_on_cross = trend_analysis.value_on_cross.to_string();
                let value_after_n_days = trend_analysis.value_after_n_days.to_string();
                vec![
                    cross_date,
                    chance_loss,
                    cross_direction_5_25,
                    change,
                    value_on_cross,
                    value_after_n_days,
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
    use anyhow::Result;

    use crate::domain::entity::close_cross_trend_analysis::VecCloseCrossTrendAnalysis;
    use crate::domain::entity::cross::VecCross;
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::stocks_crosses_pair::StocksCrossesPair;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use crate::presenter::display_cross_pattern::DisplayCrossPattern;
    use crate::presenter::view_model::vec_close_cross_trend_analysis::VecCloseCrossTrendAnalysisExt;

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
        let stocks_crosses_pair = StocksCrossesPair {
            stocks: vec_stock.as_slice(),
            crosses: crosses.as_slice(),
        };
        let vec_close_cross_trend_analysis =
            VecCloseCrossTrendAnalysis::<5>::from(&stocks_crosses_pair);
        let summary = vec_close_cross_trend_analysis.table_chart_summary(&DisplayCrossPattern::All);
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
        let summary =
            vec_close_cross_trend_analysis.table_chart_summary(&DisplayCrossPattern::GoldenOnly);
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
        let summary =
            vec_close_cross_trend_analysis.table_chart_summary(&DisplayCrossPattern::DeadOnly);
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
