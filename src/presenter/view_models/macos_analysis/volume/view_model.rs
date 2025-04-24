use crate::domain::models::macos_analysis::volume::model::MACOSAnalysisVolumes;
use crate::presenter::macos_pattern_filter::MACOSPatternFilter;
use crate::shared::custom_date_format::SLASH_DELIMITED_DATE_FORMAT;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

pub trait MACOSTrendAnalysisVolumesExt {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &MACOSPatternFilter) -> Vec<Vec<String>>;
}

impl MACOSTrendAnalysisVolumesExt for MACOSAnalysisVolumes {
    fn table_chart_header(&self) -> Vec<Vec<String>> {
        vec![vec![
            "date".to_string(),
            "direction".to_string(),
            "volume".to_string(),
        ]]
    }

    fn table_chart_rows(&self, pattern: &MACOSPatternFilter) -> Vec<Vec<String>> {
        self.par_iter()
            .filter(|trend_analysis| pattern.is_display_by_macos_pattern(&trend_analysis.pattern))
            .map(|trend_analysis| {
                let date = trend_analysis
                    .date_of_event
                    .format(SLASH_DELIMITED_DATE_FORMAT.as_str())
                    .to_string();
                let pattern = trend_analysis.pattern.to_string();
                let volume_on_macos = trend_analysis.volume.to_string();
                vec![date, pattern, volume_on_macos]
            })
            .collect()
    }
}
