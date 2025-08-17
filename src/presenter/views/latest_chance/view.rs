use chrono::NaiveDate;
use std::fmt;

use crate::domain::models::macos_analysis::close::model::LatestChance;
use crate::shared::custom_date_format::ISO8601_DATE_FORMAT;

impl fmt::Display for LatestChance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let latest_chance = NaiveDate::from(*self);
        write!(f, "{}", latest_chance.format(ISO8601_DATE_FORMAT))
    }
}
