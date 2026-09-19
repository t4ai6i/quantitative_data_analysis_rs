use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::statement::structures::jquants_api::Response;
use anyhow::{Result, bail};
use async_trait::async_trait;
use query_string_builder::QueryStringOwned;
use rayon::prelude::*;
use reqwest::Client;
use serde_json::Value;

const FINS_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";

fn has_usable_financial_values(row: &Value) -> bool {
    ["Sales", "EPS"].iter().any(|key| {
        row[*key]
            .as_str()
            .is_some_and(|value| !value.trim().is_empty())
    })
}

fn select_full_year_statements(rows: &[Value]) -> Vec<&Value> {
    rows.par_iter()
        .filter(|value| {
            value["CurPerType"].as_str().is_some_and(|s| s == "FY")
                && has_usable_financial_values(value)
        })
        .collect()
}

async fn fetch_statement_rows(api_key: &str, code: &str) -> anyhow::Result<Vec<Value>> {
    let qs = QueryStringOwned::new().with("code", code);
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
    async fn get_row_statements<'a>(
        &self,
        query: &queries::get_statements::Query<'a>,
    ) -> Result<Vec<model::RowStatement>> {
        let queries::get_statements::Query { code } = query;
        let Some(code) = code else {
            bail!("code is required for get_row_statements for JQuantsAPI");
        };
        let rows = fetch_statement_rows(&self.api_key, code).await?;
        let selected = select_full_year_statements(&rows);
        if selected.is_empty() {
            let serialized = serde_json::to_string_pretty(&rows)
                .unwrap_or_else(|_| "<failed to serialize response>".to_string());
            bail!(
                "FY statement not found in response[data]. code: {}\n{}",
                code,
                serialized
            );
        }

        Ok(selected
            .into_iter()
            .map(|value| {
                model::RowStatement::from(Response {
                    code: code.to_string(),
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
    use crate::infrastructure::repositories::statement::jquants_api::select_full_year_statements;
    use crate::shared::jquants_api::setup::Setup;
    use anyhow::Result;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn select_full_year_statements_returns_all_fy() {
        let rows = vec![
            json!({"CurPerType": "FY", "id": 1, "Sales": "1000", "EPS": "10"}),
            json!({"CurPerType": "FY", "id": 2, "Sales": "", "EPS": ""}),
            json!({"CurPerType": "FY", "id": 3, "Sales": "1200", "EPS": "12"}),
            json!({"CurPerType": "FY", "id": 4, "Sales": "", "EPS": ""}),
        ];
        let selected = select_full_year_statements(&rows);
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0]["id"], 1);
        assert_eq!(selected[1]["id"], 3);
    }

    #[tokio::test]
    async fn get_row_statements_test() -> Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_statements::Query { code: Some("8473") };
        let actual = repository.get_row_statements(&query).await?;
        let expected = model::RowStatement {
            code: "8473".to_string(),
            disclosed_date: NaiveDate::from_ymd_opt(2026, 7, 2),
            current_fiscal_year_end_date: NaiveDate::from_ymd_opt(2026, 3, 31),
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
        assert_eq!(actual.last().unwrap().clone(), expected);

        let query = queries::get_statements::Query { code: Some("????") };
        let actual = repository
            .get_row_statements(&query)
            .await
            .unwrap_err()
            .to_string();
        assert!(actual.contains("response[data] in response not found. code: ????"));

        let query = queries::get_statements::Query { code: Some("2995") };
        let actual = repository
            .get_row_statements(&query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[data] in response is empty array. code: 2995";
        assert_eq!(actual, expected);

        Ok(())
    }
}
