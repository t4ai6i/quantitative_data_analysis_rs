use crate::domain::entity::close_macos_trend_analysis::MACOSRateOfChance;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

impl MACOSRateOfChance {
    pub fn to_string(&self, display_macos_pattern: &DisplayMACOSPattern) -> String {
        let rate_of_chance = display_macos_pattern.get_rate_of_chance(self);
        format!("{:.0}%", rate_of_chance)
    }
}
