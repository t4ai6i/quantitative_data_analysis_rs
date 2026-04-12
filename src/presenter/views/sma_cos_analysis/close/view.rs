use crate::domain::models::crossover_strategy::analysis_pattern::model::AnalysisPattern;
use crate::domain::models::crossover_strategy::sma_cos::analysis_result::close::model;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::shared::custom_serde::naive_date::SLASH_DELIMITED_DATE_FORMAT;
use rayon::prelude::*;

pub trait SmaCosAnalysis {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &CrossoverPatternFilter) -> Vec<Vec<String>>;
}

impl<const N: usize> SmaCosAnalysis for model::AnalysisResults<N> {
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

    fn table_chart_rows(&self, pattern: &CrossoverPatternFilter) -> Vec<Vec<String>> {
        self.par_iter()
            .filter(|analysis_result| pattern.matches_filter(&analysis_result.pattern))
            .map(|analysis_result| {
                let chance_loss = match analysis_result.analysis_pattern {
                    AnalysisPattern::None => "❔".to_string(),
                    AnalysisPattern::GoldenChance => "✅".to_string(),
                    AnalysisPattern::DeadChance => "✅".to_string(),
                    AnalysisPattern::GoldenLoss => "❌".to_string(),
                    AnalysisPattern::DeadLoss => "❌".to_string(),
                };
                let date_of_event = analysis_result
                    .date_of_event
                    .format(SLASH_DELIMITED_DATE_FORMAT)
                    .to_string();
                let pattern = analysis_result.pattern.to_string();
                let rate_of_change = format!("{:+.3}%", analysis_result.rate_of_change);
                let close = analysis_result.close.to_string();
                let close_after_n_days = analysis_result.close_after_n_days.to_string();
                vec![
                    date_of_event,
                    chance_loss,
                    pattern,
                    rate_of_change,
                    close,
                    close_after_n_days,
                ]
            })
            .collect()
    }
}
