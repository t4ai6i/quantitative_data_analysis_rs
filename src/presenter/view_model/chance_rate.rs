use crate::domain::entity::close_macos_trend_analysis::ChanceRate;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

impl ChanceRate {
    pub fn to_string(&self, display_macos_pattern: &DisplayMACOSPattern) -> String {
        let chance_rate = display_macos_pattern.get_chance_rate(self);
        format!("{:.0}%", chance_rate)
    }
}
