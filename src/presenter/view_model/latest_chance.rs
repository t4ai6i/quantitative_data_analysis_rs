use crate::domain::entity::close_macos_trend_analysis::LatestChance;
use chrono::NaiveDate;
use std::fmt;

impl fmt::Display for LatestChance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let latest_chance = NaiveDate::from(*self);
        write!(f, "{}", latest_chance.format("%Y-%m-%d"))
    }
}
