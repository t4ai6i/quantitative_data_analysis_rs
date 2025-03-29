use crate::domain::models::macos_analysis::close::model::RateOfChance;
use crate::presenter::display_macos_pattern::DisplayMACOSPattern;

impl RateOfChance {
    pub fn to_string(&self, display_macos_pattern: &DisplayMACOSPattern) -> String {
        let rate_of_chance = display_macos_pattern.get_rate_of_chance(self);
        format!("{:.0}%", rate_of_chance)
    }
}
