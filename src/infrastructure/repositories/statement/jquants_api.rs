use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::statement::structures::jquants_api::Response;
use anyhow::{Context, bail};
use async_trait::async_trait;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;
use serde_json::Value;

const FINS_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";

fn select_latest_full_year_statement(rows: &[Value]) -> Option<&Value> {
    rows.par_iter()
        .find_last(|value| value["CurPerType"].as_str().is_some_and(|s| s == "FY"))
}

fn has_usable_financial_values(row: &Value) -> bool {
    ["Sales", "EPS"].iter().any(|key| {
        row[*key]
            .as_str()
            .is_some_and(|value| !value.trim().is_empty())
    })
}

fn select_preferred_full_year_statement(rows: &[Value]) -> Option<&Value> {
    rows.par_iter()
        .rev()
        .find_first(|value| {
            value["CurPerType"].as_str().is_some_and(|s| s == "FY")
                && has_usable_financial_values(value)
        })
        .or_else(|| select_latest_full_year_statement(rows))
}

fn select_full_year_statements(rows: &[Value]) -> Vec<&Value> {
    rows.par_iter()
        .filter(|value| value["CurPerType"].as_str().is_some_and(|s| s == "FY"))
        .collect()
}

async fn fetch_statement_rows(api_key: &str, code: &str) -> anyhow::Result<Vec<Value>> {
    let qs = QueryString::dynamic().with_value("code", code);
    let url = format!("{FINS_SUMMARY_URL}{qs}");
    let response = Client::new()
        .get(url)
        .header("x-api-key", api_key.to_string())
        .send()
        .await?;
    let response = response.json::<Value>().await?;
    let Some(rows) = response["data"].as_array() else {
        let serialized = serde_json::to_string(&response)
            .unwrap_or_else(|_| "<failed to serialize>".to_string());
        bail!(
            "response[data] in response not found. code: {} response: {}",
            code,
            serialized
        );
    };
    if rows.is_empty() {
        bail!("response[data] in response is empty array. code: {}", code);
    }

    Ok(rows.to_vec())
}

#[async_trait]
impl repository::Statement for JQuantsAPI {
    async fn get_row_statement<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> anyhow::Result<model::RowStatement> {
        let rows = fetch_statement_rows(&self.api_key, query.code).await?;
        let selected = select_preferred_full_year_statement(&rows).with_context(|| {
            let serialized = serde_json::to_string_pretty(&rows)
                .unwrap_or_else(|_| "<failed to serialize response>".to_string());
            format!(
                "FY statement not found in response[data]. code: {}\n{}",
                query.code, serialized
            )
        })?;

        Ok(From::from(Response {
            code: query.code.to_string(),
            value: selected,
        }))
    }

    async fn get_row_full_year_statements<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> anyhow::Result<Vec<model::RowStatement>> {
        let rows = fetch_statement_rows(&self.api_key, query.code).await?;
        let selected = select_full_year_statements(&rows);
        if selected.is_empty() {
            let serialized = serde_json::to_string_pretty(&rows)
                .unwrap_or_else(|_| "<failed to serialize response>".to_string());
            bail!(
                "FY statement not found in response[data]. code: {}\n{}",
                query.code,
                serialized
            );
        }

        Ok(selected
            .into_iter()
            .map(|value| {
                model::RowStatement::from(Response {
                    code: query.code.to_string(),
                    value,
                })
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::models::statement::model;
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::infrastructure::repositories::statement::jquants_api::{
        select_full_year_statements, select_latest_full_year_statement,
        select_preferred_full_year_statement,
    };
    use crate::shared::jquants_api::setup::Setup;
    use anyhow::Result;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn select_latest_full_year_statement_prefers_latest_fy() {
        let rows = vec![
            json!({"CurPerType": "Q1", "id": 1}),
            json!({"CurPerType": "FY", "id": 2}),
            json!({"CurPerType": "FY", "id": 3}),
        ];
        let selected = select_latest_full_year_statement(&rows).expect("record should be selected");
        assert_eq!(selected["id"], 3);
    }

    #[test]
    fn select_latest_full_year_statement_returns_none_when_fy_missing() {
        let rows = vec![
            json!({"CurPerType": "Q1", "id": 1}),
            json!({"CurPerType": "Q2", "id": 2}),
        ];
        let selected = select_latest_full_year_statement(&rows);
        assert!(selected.is_none());
    }

    #[test]
    fn select_full_year_statements_returns_all_fy() {
        let rows = vec![
            json!({"CurPerType": "Q1", "id": 1}),
            json!({"CurPerType": "FY", "id": 2}),
            json!({"CurPerType": "FY", "id": 3}),
        ];
        let selected = select_full_year_statements(&rows);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0]["id"], 2);
        assert_eq!(selected[1]["id"], 3);
    }

    #[test]
    fn select_preferred_full_year_statement_prefers_latest_usable_fy() {
        let rows = vec![
            json!({"CurPerType": "FY", "id": 1, "Sales": "1000", "EPS": "10"}),
            json!({"CurPerType": "FY", "id": 2, "Sales": "", "EPS": ""}),
            json!({"CurPerType": "FY", "id": 3, "Sales": "1200", "EPS": "12"}),
            json!({"CurPerType": "FY", "id": 4, "Sales": "", "EPS": ""}),
        ];
        let selected =
            select_preferred_full_year_statement(&rows).expect("record should be selected");
        assert_eq!(selected["id"], 3);
    }

    #[test]
    fn select_preferred_full_year_statement_falls_back_to_latest_fy() {
        let rows = vec![
            json!({"CurPerType": "FY", "id": 1, "Sales": "", "EPS": ""}),
            json!({"CurPerType": "FY", "id": 2, "Sales": "", "EPS": ""}),
        ];
        let selected =
            select_preferred_full_year_statement(&rows).expect("record should be selected");
        assert_eq!(selected["id"], 2);
    }

    #[tokio::test]
    async fn get_row_statement_test() -> Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_statement::Query { code: "8473" };
        let actual = repository.get_row_statement(&query).await?;
        let expected = model::RowStatement {
            code: "8473".to_string(),
            disclosed_date: NaiveDate::from_ymd_opt(2026, 7, 2),
            current_fiscal_year_end_date: NaiveDate::from_ymd_opt(2026, 6, 30),
            eps: Some(666.82),
            bps: Some(2776.99),
            annual_dividend_forecast: None,
            net_sales: Some(1896607000000.0),
            opp: None,
            orp: None,
            profit: Some(427577000000.0),
            equity: Some(2413363000000.0),
            total_assets: Some(38290797000000.0),
        };
        assert_eq!(actual, expected);

        let query = queries::get_statement::Query { code: "????" };
        let actual = repository
            .get_row_statement(&query)
            .await
            .unwrap_err()
            .to_string();
        assert!(actual.contains("response[data] in response not found. code: ????"));

        let query = queries::get_statement::Query { code: "2995" };
        let actual = repository
            .get_row_statement(&query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[data] in response is empty array. code: 2995";
        assert_eq!(actual, expected);

        Ok(())
    }
}
