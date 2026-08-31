use crate::domain::repositories::market_calendar::queries::get_previous_business_day::Query;
use crate::domain::repositories::market_calendar::repository;
use crate::infrastructure::jquants_api::JQuantsAPI;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use chrono::{Days, NaiveDate};
use query_string_builder::QueryString;
use reqwest::Client;
use serde_json::Value;

const MARKETS_CALENDAR_URL: &str = "https://api.jquants.com/v2/markets/calendar";
const CALENDAR_LOOKBACK_DAYS: u64 = 31;

fn is_trading_day(hol_div: &str) -> bool {
    matches!(hol_div, "1" | "2")
}

fn select_previous_business_day(rows: &[Value], target_date: NaiveDate) -> Option<NaiveDate> {
    rows.iter()
        .filter_map(|row| {
            let date_str = row["Date"].as_str()?;
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").ok()?;
            let hol_div = row["HolDiv"].as_str()?;
            if is_trading_day(hol_div) && date <= target_date {
                Some(date)
            } else {
                None
            }
        })
        .max()
}

fn build_query(from: NaiveDate, to: NaiveDate, pagination_key: Option<&str>) -> String {
    let mut qs = QueryString::dynamic()
        .with_value("from", from.to_string())
        .with_value("to", to.to_string());

    if let Some(pagination_key) = pagination_key {
        qs = qs.with_value("pagination_key", pagination_key);
    }

    qs.to_string()
}

#[async_trait]
impl repository::MarketCalendar for JQuantsAPI {
    async fn get_previous_business_day(&self, query: &Query) -> Result<NaiveDate> {
        let from_date = query
            .target_date
            .checked_sub_days(Days::new(CALENDAR_LOOKBACK_DAYS))
            .unwrap_or(query.target_date);

        let mut rows = Vec::new();
        let mut pagination_key: Option<String> = None;

        loop {
            let query_string = build_query(from_date, query.target_date, pagination_key.as_deref());
            let url = format!("{}{}", MARKETS_CALENDAR_URL, query_string);
            let response = Client::new()
                .get(url)
                .header("x-api-key", self.api_key.to_string())
                .send()
                .await?;
            let response = response.json::<Value>().await?;
            let Some(page_rows) = response["data"].as_array() else {
                bail!(
                    "response[data] in response not found. target_date = {}",
                    query.target_date
                );
            };

            rows.extend(page_rows.iter().cloned());

            pagination_key = response["pagination_key"]
                .as_str()
                .filter(|value| !value.is_empty())
                .map(ToOwned::to_owned);

            if pagination_key.is_none() {
                break;
            }
        }

        select_previous_business_day(&rows, query.target_date).with_context(|| {
            format!(
                "Trading day not found for target_date = {}",
                query.target_date
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::market_calendar::queries::get_previous_business_day::Query;
    use crate::domain::repositories::market_calendar::repository::MarketCalendar;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::infrastructure::repositories::market_calendar::jquants_api::{
        build_query, select_previous_business_day,
    };
    use crate::shared::jquants_api::setup;
    use anyhow::Result;
    use chrono::{NaiveDate, Utc};
    use chrono_tz::Asia::Tokyo;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn select_previous_business_day_returns_latest_trading_day_at_or_before_target() {
        let rows = vec![
            json!({"Date": "2026-08-26", "HolDiv": "1"}),
            json!({"Date": "2026-08-27", "HolDiv": "0"}),
            json!({"Date": "2026-08-28", "HolDiv": "2"}),
            json!({"Date": "2026-08-29", "HolDiv": "0"}),
        ];
        let target_date = NaiveDate::from_ymd_opt(2026, 8, 29).unwrap();

        let actual = select_previous_business_day(&rows, target_date);

        assert_eq!(actual, Some(NaiveDate::from_ymd_opt(2026, 8, 28).unwrap()));
    }

    #[test]
    fn select_previous_business_day_ignores_non_trading_days_and_invalid_dates() {
        let rows = vec![
            json!({"Date": "not-a-date", "HolDiv": "1"}),
            json!({"Date": "2026-08-26", "HolDiv": "0"}),
            json!({"Date": "2026-08-27", "HolDiv": "2"}),
        ];
        let target_date = NaiveDate::from_ymd_opt(2026, 8, 27).unwrap();

        let actual = select_previous_business_day(&rows, target_date);

        assert_eq!(actual, Some(NaiveDate::from_ymd_opt(2026, 8, 27).unwrap()));
    }

    #[test]
    fn select_previous_business_day_returns_none_when_no_trading_day_exists() {
        let rows = vec![
            json!({"Date": "2026-08-26", "HolDiv": "0"}),
            json!({"Date": "2026-08-27", "HolDiv": "0"}),
        ];
        let target_date = NaiveDate::from_ymd_opt(2026, 8, 27).unwrap();

        let actual = select_previous_business_day(&rows, target_date);

        assert_eq!(actual, None);
    }

    #[test]
    fn build_query_includes_from_to_and_pagination_key() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 29).unwrap();

        let actual = build_query(from, to, Some("next-page-token"));

        assert_eq!(
            actual,
            "?from=2026-08-01&to=2026-08-29&pagination_key=next-page-token"
        );
    }

    #[test]
    fn build_query_omits_pagination_key_when_absent() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 29).unwrap();

        let actual = build_query(from, to, None);

        assert_eq!(actual, "?from=2026-08-01&to=2026-08-29");
    }

    #[tokio::test]
    async fn dbg_live_markets_calendar_response() -> Result<()> {
        let token = setup::Setup::run()?;
        let jquants_api = JQuantsAPI::new(token)?;
        let now = Utc::now();
        let target_date = now.with_timezone(&Tokyo).date_naive();
        let query = Query { target_date };
        let actual = jquants_api.get_previous_business_day(&query).await?;
        dbg!(actual);
        assert!(actual <= target_date);
        Ok(())
    }
}
