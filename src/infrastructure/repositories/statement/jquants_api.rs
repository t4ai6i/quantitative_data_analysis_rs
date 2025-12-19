use anyhow::{Context, bail};
use async_trait::async_trait;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;

use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::statement::structures::jquants_api::Response;

const STATEMENT_URL: &str = "https://api.jquants.com/v1/fins/statements";

#[async_trait]
impl repository::Statement for JQuantsAPI {
    async fn get_row_statement<'a>(
        &self,
        query: &queries::get_statement::Query<'a>,
    ) -> anyhow::Result<model::RowStatement> {
        let qs = QueryString::dynamic().with_value("code", query.code);
        let url = format!("{STATEMENT_URL}{qs}");
        let response = Client::new()
            .get(url)
            .bearer_auth(self.id_token.clone())
            .send()
            .await?;
        let response = &response.json::<serde_json::Value>().await?;
        let Some(response) = response["statements"].as_array() else {
            bail!(
                "response[statements] in response not found. code: {}",
                query.code
            );
        };
        if response.is_empty() {
            bail!(
                "response[statements] in response is empty array. code: {}",
                query.code
            );
        }
        response
            .par_iter()
            .filter_map(|value| {
                // TypeOfCurrentPeriodはFY(Fiscal Year/事業年度)のみを対象とする
                value["TypeOfCurrentPeriod"]
                    .as_str()
                    .filter(|&str| str.eq("FY"))?;
                Some(From::from(Response {
                    code: query.code.to_string(),
                    value,
                }))
            })
            // API Docの以下の記述に従い、取得した配列データの最後尾を取得する。
            // 「DisclosureNumber: APIから出力されるjsonは開示番号で昇順に並んでいます。」
            .reduce_with(|_, b| b)
            .with_context(|| {
                let response = serde_json::to_string_pretty(response).unwrap();
                format!(
                    "struct model::Statement cannot be constructed. code: {}\n{}",
                    query.code, response
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use bytestring::ByteString;
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use rstest::*;

    use crate::domain::models::statement::model;
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::jquants_api::JQuantsAPI;
    use crate::shared::jquants_api::setup::Setup;

    #[fixture]
    async fn setup() -> Result<ByteString> {
        Setup::run().await
    }

    #[rstest]
    #[tokio::test]
    async fn get_row_statement_test(#[future] setup: Result<ByteString>) -> Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token)?;
        let query = queries::get_statement::Query { code: "8473" };
        let actual = repository.get_row_statement(&query).await?;
        let expected = model::RowStatement {
            code: "8473".to_string(),
            disclosed_date: NaiveDate::from_ymd_opt(2025, 5, 9),
            eps: Some(536.09),
            bps: Some(4162.73),
            net_sales: Some(1443733000000.0),
            opp: None,
            orp: None,
            profit: Some(162120000000.0),
            equity: Some(1763793000000.0),
            total_assets: Some(32113430000000.0),
        };
        assert_eq!(actual, expected);

        let query = queries::get_statement::Query { code: "????" };
        let actual = repository
            .get_row_statement(&query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[statements] in response not found. code: ????";
        assert_eq!(actual, expected);

        let query = queries::get_statement::Query { code: "2995" };
        let actual = repository
            .get_row_statement(&query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[statements] in response is empty array. code: 2995";
        assert_eq!(actual, expected);

        Ok(())
    }
}
