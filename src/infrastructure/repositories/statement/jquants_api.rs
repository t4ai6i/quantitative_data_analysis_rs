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

#[async_trait]
impl repository::Statement for JQuantsAPI {
    async fn get_row_statement<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> anyhow::Result<model::RowStatement> {
        let qs = QueryString::dynamic().with_value("code", query.code);
        let url = format!("{FINS_SUMMARY_URL}{qs}");
        let response = Client::new()
            .get(url)
            .header("x-api-key", self.api_key.to_string())
            .send()
            .await?;
        let response = &response.json::<Value>().await?;
        let Some(response) = response["data"].as_array() else {
            bail!("response[data] in response not found. code: {}", query.code);
        };
        if response.is_empty() {
            bail!(
                "response[data] in response is empty array. code: {}",
                query.code
            );
        }
        let selected = select_latest_full_year_statement(response).with_context(|| {
            let serialized = serde_json::to_string_pretty(response)
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
}

#[cfg(test)]
mod tests {
    use crate::domain::models::statement::model;
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::infrastructure::repositories::statement::jquants_api::select_latest_full_year_statement;
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

    #[tokio::test]
    async fn get_row_statement_test() -> Result<()> {
        let token = Setup::run()?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_statement::Query { code: "8473" };
        let actual = repository.get_row_statement(&query).await?;
        let expected = model::RowStatement {
            code: "8473".to_string(),
            disclosed_date: NaiveDate::from_ymd_opt(2026, 7, 2),
            eps: Some(666.82),
            bps: Some(2776.99),
            net_sales: Some(1_896_607_000_000.0),
            opp: None,
            orp: None,
            profit: Some(427_577_000_000.0),
            equity: Some(2_413_363_000_000.0),
            total_assets: Some(38_290_797_000_000.0),
        };
        assert_eq!(actual, expected);

        let query = queries::get_statement::Query { code: "????" };
        let actual = repository
            .get_row_statement(&query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[data] in response not found. code: ????";
        assert_eq!(actual, expected);

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
