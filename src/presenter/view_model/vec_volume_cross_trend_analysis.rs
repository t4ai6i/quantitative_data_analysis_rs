use itertools::Itertools;

use crate::domain::entity::volume_cross_trend_analysis::VecVolumeCrossTrendAnalysis;
use crate::presenter::display_cross_pattern::DisplayCrossPattern;

pub trait VecVolumeCrossTrendAnalysisExt {
    fn table_chart_header(&self) -> Vec<Vec<String>>;
    fn table_chart_rows(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>>;
}

impl VecVolumeCrossTrendAnalysisExt for VecVolumeCrossTrendAnalysis {
    fn table_chart_header(&self) -> Vec<Vec<String>> {
        vec![vec![
            "date".to_string(),
            "direction".to_string(),
            "volume".to_string(),
        ]]
    }

    fn table_chart_rows(&self, pattern: &DisplayCrossPattern) -> Vec<Vec<String>> {
        self.0
            .iter()
            .filter(|trend_analysis| {
                pattern.is_display_by_cross_direction_type(&trend_analysis.r#type)
            })
            .map(|trend_analysis| {
                let date = trend_analysis.date.format("%Y/%m/%d").to_string();
                let r#type = trend_analysis.r#type.to_string();
                let value_on_cross = trend_analysis.value_on_cross.to_string();
                vec![date, r#type, value_on_cross]
            })
            .collect_vec()
    }
}
