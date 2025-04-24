use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serializer};
use std::sync::LazyLock;

pub static SLASH_DELIMITED_DATE_FORMAT: LazyLock<String> = LazyLock::new(|| {
    let date_format = "%Y/%m/%d";
    date_format.to_string()
});
pub static ISO8601_DATE_FORMAT: LazyLock<String> = LazyLock::new(|| {
    let date_format = "%Y-%m-%d";
    date_format.to_string()
});

pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = format!("{}", date.format(ISO8601_DATE_FORMAT.as_str()));
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, ISO8601_DATE_FORMAT.as_str()).map_err(serde::de::Error::custom)
}
