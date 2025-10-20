use rayon::prelude::*;

use crate::domain::models::macos_analysis::volume::model;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::shared::custom_date_format::primitive::SLASH_DELIMITED_DATE_FORMAT;

pub trait MACOSTrendAnalysisVolumes {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &CrossoverPatternFilter) -> Vec<Vec<String>>;
}

impl MACOSTrendAnalysisVolumes for model::MACOSAnalysisVolumes {
    fn table_chart_header(&self) -> Vec<Vec<String>> {
        vec![vec![
            "date".to_string(),
            "direction".to_string(),
            "volume".to_string(),
        ]]
    }

    fn table_chart_rows(&self, pattern: &CrossoverPatternFilter) -> Vec<Vec<String>> {
        self.par_iter()
            .filter(|trend_analysis| pattern.matches_filter(&trend_analysis.pattern))
            .map(|trend_analysis| {
                let date = trend_analysis
                    .date_of_event
                    .format(SLASH_DELIMITED_DATE_FORMAT)
                    .to_string();
                let pattern = trend_analysis.pattern.to_string();
                let volume_on_macos = trend_analysis.volume.to_string();
                vec![date, pattern, volume_on_macos]
            })
            .collect()
    }
}
