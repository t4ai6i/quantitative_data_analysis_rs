use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serializer};

pub const SLASH_DELIMITED_DATE_FORMAT: &str = "%Y/%m/%d";
pub const ISO8601_DATE_FORMAT: &str = "%Y-%m-%d";

pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(ISO8601_DATE_FORMAT));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, ISO8601_DATE_FORMAT).map_err(serde::de::Error::custom)
}
