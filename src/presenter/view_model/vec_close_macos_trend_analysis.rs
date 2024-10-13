use itertools::Itertools;

use crate::domain::entity::chance_loss::ChanceLoss;
use crate::domain::entity::close_macos_trend_analysis::VecCloseMACOSTrendAnalysis;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

pub trait VecCloseMACOSTrendAnalysisExt {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>>;
    fn table_chart_summary(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>>;
}

impl<const N: usize> VecCloseMACOSTrendAnalysisExt for VecCloseMACOSTrendAnalysis<N> {
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

    fn table_chart_rows(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>> {
        self.vec_close_macos_trend_analysis
            .iter()
            .filter(|trend_analysis| pattern.is_display_by_macos_type(&trend_analysis.r#type))
            .map(|trend_analysis| {
                let chance_loss = match trend_analysis.chance_loss {
                    ChanceLoss::None => "❔".to_string(),
                    ChanceLoss::GoldenChance => "✅".to_string(),
                    ChanceLoss::DeadChance => "✅".to_string(),
                    ChanceLoss::GoldenLoss => "❌".to_string(),
                    ChanceLoss::DeadLoss => "❌".to_string(),
                };
                let macos_date = trend_analysis.date.format("%Y/%m/%d").to_string();
                let macos_5_25 = trend_analysis.r#type.to_string();
                let change = format!("{:+.3}%", trend_analysis.rate_of_change);
                let close_on_macos = trend_analysis.close_on_macos.to_string();
                let close_after_n_days = trend_analysis.close_after_n_days.to_string();
                vec![
                    macos_date,
                    chance_loss,
                    macos_5_25,
                    change,
                    close_on_macos,
                    close_after_n_days,
                ]
            })
            .collect_vec()
    }

    fn table_chart_summary(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>> {
        let chance_rate = match pattern {
            DisplayMACOSPattern::All => {
                format!("{:.0}%", self.chance_rate.all)
            }
            DisplayMACOSPattern::GoldenOnly => {
                format!("{:.0}%", self.chance_rate.golden_only)
            }
            DisplayMACOSPattern::DeadOnly => {
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

    use crate::domain::entity::close_macos_trend_analysis::VecCloseMACOSTrendAnalysis;
    use crate::domain::entity::macos::VecMACOS;
    use crate::domain::entity::sma::{SMAListPair, VecSMA};
    use crate::domain::entity::stocks_macoses_pair::StocksMACOSESPair;
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::stock_repository::data_format::csv::Csv;
    use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
    use crate::presenter::view_model::vec_close_macos_trend_analysis::VecCloseMACOSTrendAnalysisExt;

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
        let VecMACOS(macoses) = VecMACOS::from(sma_list_pair);
        let stocks_macoses_pair = StocksMACOSESPair {
            stocks: vec_stock.as_slice(),
            macoses: macoses.as_slice(),
        };
        let vec_close_macos_trend_analysis =
            VecCloseMACOSTrendAnalysis::<5>::from(&stocks_macoses_pair);
        let summary = vec_close_macos_trend_analysis.table_chart_summary(&DisplayMACOSPattern::All);
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
            vec_close_macos_trend_analysis.table_chart_summary(&DisplayMACOSPattern::GoldenOnly);
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
            vec_close_macos_trend_analysis.table_chart_summary(&DisplayMACOSPattern::DeadOnly);
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
