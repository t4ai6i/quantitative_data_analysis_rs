use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serializer};

pub const SLASH_DELIMITED_DATE_FORMAT: &str = "%Y/%m/%d";
pub const ISO8601_DATE_FORMAT: &str = "%Y-%m-%d";

pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => {
            let string = format!("{}", date.format(ISO8601_DATE_FORMAT));
            serializer.serialize_str(&string)
        }
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let option: Option<String> = Option::deserialize(deserializer)?;
    let Some(string) = option else {
        return Ok(None);
    };
    let date = NaiveDate::parse_from_str(&string, ISO8601_DATE_FORMAT)
        .map_err(serde::de::Error::custom)?;
    Ok(Some(date))
}
