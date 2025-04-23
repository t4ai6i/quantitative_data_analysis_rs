use crate::domain::models::macos_analysis::close::model::LatestChance;
use chrono::NaiveDate;
use std::fmt;

impl fmt::Display for LatestChance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let latest_chance = NaiveDate::from(*self);
        write!(f, "{}", latest_chance.format("%Y-%m-%d"))
    }
}
