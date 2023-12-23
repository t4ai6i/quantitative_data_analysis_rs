use crate::domain::entity::trend_analysis::ChanceRate;
use crate::presenter::trend_analysis_presenter::DisplayCrossPattern;

impl ChanceRate {
    pub fn to_string(&self, display_cross_pattern: &DisplayCrossPattern) -> String {
        let chance_rate = display_cross_pattern.get_chance_rate(self);
        format!("{:.0}%", chance_rate)
    }
}
