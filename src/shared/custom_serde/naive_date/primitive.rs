use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serializer};

use crate::shared::custom_serde::naive_date::ISO8601_DATE_FORMAT;

pub fn serialize<S>(date: &NaiveDate, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let date_string = format!("{}", date.format(ISO8601_DATE_FORMAT));
    s.serialize_str(&date_string)
}

pub fn deserialize<'de, D>(d: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let date_string = String::deserialize(d)?;
    NaiveDate::parse_from_str(&date_string, ISO8601_DATE_FORMAT).map_err(serde::de::Error::custom)
}
