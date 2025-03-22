use itertools::Itertools;

use crate::domain::entity::volume_macos_trend_analysis::VecVolumeMACOSTrendAnalysis;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

pub trait VecVolumeMACOSTrendAnalysisExt {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>>;
}

impl VecVolumeMACOSTrendAnalysisExt for VecVolumeMACOSTrendAnalysis {
    fn table_chart_header(&self) -> Vec<Vec<String>> {
        vec![vec![
            "date".to_string(),
            "direction".to_string(),
            "volume".to_string(),
        ]]
    }

    fn table_chart_rows(&self, pattern: &DisplayMACOSPattern) -> Vec<Vec<String>> {
        self.0
            .iter()
            .filter(|trend_analysis| pattern.is_display_by_macos_pattern(&trend_analysis.pattern))
            .map(|trend_analysis| {
                let date = trend_analysis.date.format("%Y/%m/%d").to_string();
                let pattern = trend_analysis.pattern.to_string();
                let volume_on_macos = trend_analysis.volume_on_macos.to_string();
                vec![date, pattern, volume_on_macos]
            })
            .collect_vec()
    }
}
