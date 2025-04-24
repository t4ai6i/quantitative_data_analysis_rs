use crate::domain::models::macos::model::AnalysisPattern;
use crate::domain::models::macos_analysis::close::model::MACOSAnalysisCloses;
use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::shared::custom_date_format::SLASH_DELIMITED_DATE_FORMAT;
use rayon::prelude::*;

pub trait MACOSAnalysisClosesExt {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &MACOSPatternFilter) -> Vec<Vec<String>>;
}

impl<const N: usize> MACOSAnalysisClosesExt for MACOSAnalysisCloses<N> {
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

    fn table_chart_rows(&self, pattern: &MACOSPatternFilter) -> Vec<Vec<String>> {
        self.par_iter()
            .filter(|macos_analysis_close| {
                pattern.is_display_by_macos_pattern(&macos_analysis_close.pattern)
            })
            .map(|macos_analysis_close| {
                let chance_loss = match macos_analysis_close.analysis_pattern {
                    AnalysisPattern::None => "❔".to_string(),
                    AnalysisPattern::GoldenChance => "✅".to_string(),
                    AnalysisPattern::DeadChance => "✅".to_string(),
                    AnalysisPattern::GoldenLoss => "❌".to_string(),
                    AnalysisPattern::DeadLoss => "❌".to_string(),
                };
                let date_of_event = macos_analysis_close
                    .date_of_event
                    .format(SLASH_DELIMITED_DATE_FORMAT.as_str())
                    .to_string();
                let pattern = macos_analysis_close.pattern.to_string();
                let rate_of_change = format!("{:+.3}%", macos_analysis_close.rate_of_change);
                let close = macos_analysis_close.close.to_string();
                let close_after_n_days = macos_analysis_close.close_after_n_days.to_string();
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
