use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::statement::structures::jquants_api::Response;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use query_string_builder::QueryString;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashSet;
use std::future::Future;

const FINS_SUMMARY_URL: &str = "https://api.jquants.com/v2/fins/summary";

async fn collect_pages<F, Fut>(mut fetch_page: F) -> Result<Vec<Value>>
where
    F: FnMut(Option<String>) -> Fut,
    Fut: Future<Output = Result<Value>>,
{
    let mut rows = Vec::new();
    let mut next = None;
    let mut seen = HashSet::new();
    loop {
        let response = fetch_page(next).await?;
        let page = response
            .get("data")
            .and_then(Value::as_array)
            .context("response data must be an array")?;
        rows.extend(page.iter().cloned());
        next = match response.get("pagination_key") {
            None => break,
            Some(Value::String(key)) if !key.is_empty() => {
                if !seen.insert(key.clone()) {
                    bail!("repeated pagination key");
                }
                Some(key.clone())
            }
            Some(_) => bail!("invalid pagination key"),
        };
    }
    Ok(rows)
}

fn convert_rows(code: &str, rows: &[Value]) -> Result<Vec<model::RowStatement>> {
    let mut statements = rows
        .iter()
        .map(|value| {
            model::RowStatement::try_from(Response {
                code: code.to_owned(),
                value,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    statements.sort_by_key(|row| {
        row.disclosure
            .as_ref()
            .map(|disclosure| (disclosure.disclosed_date, disclosure.disclosed_time))
    });
    Ok(statements)
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
        let client = Client::new();
        let rows = collect_pages(|pagination_key| {
            let client = &client;
            async move {
                let url = {
                    let mut query = QueryString::new().with("code", code);
                    if let Some(key) = pagination_key.as_deref() {
                        query = query.with("pagination_key", key);
                    }
                    format!("{FINS_SUMMARY_URL}{query}")
                };
                Ok(client
                    .get(url)
                    .header("x-api-key", self.api_key.to_string())
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?)
            }
        })
        .await
        .with_context(|| format!("failed to fetch disclosures for code {code}"))?;

        convert_rows(code, &rows)
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_pages, convert_rows};
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::statement::structures::internal::tsv::Structure;
    use anyhow::anyhow;
    use chrono::NaiveDate;
    use serde_json::{Value, json};

    fn disclosure(date: &str, time: &str, number: &str, kind: &str) -> Value {
        json!({
            "Code": "12340", "DiscDate": date, "DiscTime": time, "DiscNo": number,
            "DocType": "3QFinancialStatements_Consolidated_IFRS", "CurPerType": kind,
            "CurPerSt": "2025-04-01", "CurPerEn": "2025-12-31",
            "CurFYSt": "2025-04-01", "CurFYEn": "2026-03-31",
            "NxtFYSt": "", "NxtFYEn": "", "OP": "90", "FOP": "120",
            "NxFOP": "", "NCOP": "70"
        })
    }

    #[tokio::test]
    async fn collects_every_page_in_order() {
        let rows = collect_pages(|key| async move {
            match key.as_deref() {
                None => Ok(json!({"data": [{"id": 1}], "pagination_key": "next"})),
                Some("next") => Ok(json!({"data": [{"id": 2}]})),
                _ => Err(anyhow!("unexpected key")),
            }
        })
        .await
        .unwrap();
        assert_eq!(rows, vec![json!({"id": 1}), json!({"id": 2})]);
    }

    #[tokio::test]
    async fn rejects_failed_or_invalid_later_pages() {
        for invalid in [
            json!({"data": null}),
            json!({"data": [], "pagination_key": 42}),
            json!({"data": [], "pagination_key": null}),
            json!({"data": [], "pagination_key": ""}),
        ] {
            assert!(
                collect_pages(|key| {
                    let invalid = invalid.clone();
                    async move {
                        if key.is_none() {
                            Ok(json!({"data": [{"id": 1}], "pagination_key": "next"}))
                        } else {
                            Ok(invalid)
                        }
                    }
                })
                .await
                .is_err()
            );
        }
        assert!(
            collect_pages(|key| async move {
                if key.is_none() {
                    Ok(json!({"data": [{"id": 1}], "pagination_key": "next"}))
                } else {
                    Err(anyhow!("HTTP 429"))
                }
            })
            .await
            .is_err()
        );
        assert!(
            collect_pages(|_| async { Ok(json!({"data": [{"id": 1}], "pagination_key": "same"})) })
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn preserves_disclosures_across_pages_and_tsv() {
        let quarter = disclosure("2026-02-01", "15:00:00", "1", "3Q");
        let earlier = disclosure("2026-02-01", "09:00:00", "9", "2Q");
        let mut annual = disclosure("2026-05-01", "15:00:00", "2", "FY");
        annual["DocType"] = json!("FYFinancialStatements_Consolidated_JP");
        annual["CurPerEn"] = json!("2026-03-31");
        annual["NxtFYSt"] = json!("2026-04-01");
        annual["NxtFYEn"] = json!("2027-03-31");
        annual["FOP"] = json!("");
        annual["NxFOP"] = json!("200");
        let mut revision = disclosure("2026-06-01", "15:00:00", "3", "FY");
        revision["DocType"] = json!("EarnForecastRevision");
        revision["CurFYSt"] = json!("2026-04-01");
        revision["CurFYEn"] = json!("2027-03-31");
        revision["OP"] = json!("");
        revision["FOP"] = json!("210");
        let pages = [
            json!({"data": [quarter, annual], "pagination_key": "next"}),
            json!({"data": [earlier, revision]}),
        ];
        let rows = collect_pages(|key| {
            let page = pages[usize::from(key.is_some())].clone();
            async move { Ok(page) }
        })
        .await
        .unwrap();
        let statements = convert_rows("1234", &rows).unwrap();
        let disclosures: Vec<_> = statements
            .iter()
            .map(|row| row.disclosure.as_ref().unwrap())
            .collect();
        assert_eq!(
            disclosures
                .iter()
                .map(|d| d.disclosure_number.as_str())
                .collect::<Vec<_>>(),
            ["9", "1", "2", "3"]
        );
        assert_eq!(disclosures[1].current_period_type.as_deref(), Some("3Q"));
        assert_eq!(
            disclosures[1].current_period_start,
            NaiveDate::from_ymd_opt(2025, 4, 1)
        );
        assert_eq!(
            disclosures[1].current_period_end,
            NaiveDate::from_ymd_opt(2025, 12, 31)
        );
        assert_eq!(disclosures[1].operating_profit, Some(90.0));
        assert_eq!(
            disclosures[1].operating_profit_period,
            crate::domain::models::statement::model::OperatingProfitPeriod::Unspecified
        );
        assert_eq!(disclosures[1].non_consolidated_operating_profit, Some(70.0));
        assert_eq!(
            disclosures[2].next_fiscal_year_start,
            disclosures[3].current_fiscal_year_start
        );
        assert_eq!(
            disclosures[2].next_fiscal_year_end,
            disclosures[3].current_fiscal_year_end
        );
        assert_eq!(disclosures[2].next_forecast_operating_profit, Some(200.0));
        assert_eq!(disclosures[3].forecast_operating_profit, Some(210.0));
        assert_eq!(disclosures[3].operating_profit, None);

        let mut writer = csv::WriterBuilder::new()
            .delimiter(b'\t')
            .from_writer(Vec::new());
        for row in statements.clone() {
            writer.serialize(Structure::try_from(row).unwrap()).unwrap();
        }
        let bytes = writer.into_inner().unwrap();
        let restored = Structure::process_tabular_data(&bytes, true).unwrap();
        assert_eq!(restored, statements);
        let mut mismatched = Structure::try_from(statements[1].clone()).unwrap();
        mismatched.opp = Some(999.0);
        assert!(
            crate::domain::models::statement::model::RowStatement::try_from(mismatched).is_err()
        );
        let mut mismatched = Structure::try_from(statements[1].clone()).unwrap();
        mismatched.code = "9999".into();
        assert!(
            crate::domain::models::statement::model::RowStatement::try_from(mismatched).is_err()
        );
        let serialized = serde_json::to_value(disclosures[1]).unwrap();
        assert_eq!(serialized["operating_profit_period"], "unspecified");
        assert!(
            serialized
                .get("derived_single_quarter_operating_profit")
                .is_none()
        );
    }

    #[test]
    fn rejects_invalid_disclosure_fields() {
        let valid = disclosure("2026-02-01", "15:00:00", "1", "3Q");
        for (field, invalid) in [
            ("DiscTime", json!("bad")),
            ("DiscDate", json!("bad")),
            ("DiscNo", json!("")),
            ("Code", json!("99990")),
            ("OP", json!("NaN")),
            ("FOP", json!("bad")),
            ("CurPerEn", json!("bad")),
        ] {
            let mut row = valid.clone();
            row[field] = invalid;
            assert!(convert_rows("1234", &[row]).is_err(), "{field}");
        }
        let mut missing = valid;
        missing["FOP"] = Value::Null;
        assert_eq!(
            convert_rows("1234", &[missing]).unwrap()[0]
                .disclosure
                .as_ref()
                .unwrap()
                .forecast_operating_profit,
            None
        );
    }
}
