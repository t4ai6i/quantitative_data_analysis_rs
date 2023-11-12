use crate::domain::entity::chance_loss::ChanceLoss;
use crate::domain::entity::trend_analysis::VecTrendAnalysis;
use itertools::Itertools;
use std::ops::Mul;

pub trait VecTrendAnalysisExt {
    fn table_chart_rows(&self) -> Vec<Vec<String>>;
}

impl<const N: usize> VecTrendAnalysisExt for VecTrendAnalysis<N> {
    fn table_chart_rows(&self) -> Vec<Vec<String>> {
        self.0
            .iter()
            .map(|trend_analysis| {
                let chance_loss = match trend_analysis.chance_loss {
                    ChanceLoss::None => "".to_string(),
                    ChanceLoss::GoldenChance => "✅".to_string(),
                    ChanceLoss::DeadChance => "✅".to_string(),
                    ChanceLoss::GoldenLoss => "❌".to_string(),
                    ChanceLoss::DeadLoss => "❌".to_string(),
                };
                vec![
                    trend_analysis.cross_date.format("%Y/%m/%d").to_string(),
                    chance_loss,
                    trend_analysis.cross_direction_5_25.0.to_string(),
                    format!("{:+.3}%", trend_analysis.change.mul(100.0)),
                    trend_analysis.close_on_cross.to_string(),
                    trend_analysis.close_after_n_days.to_string(),
                ]
            })
            .collect_vec()
    }
}
