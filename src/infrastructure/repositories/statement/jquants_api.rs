use anyhow::{bail, Context};
use async_trait::async_trait;
use query_string_builder::QueryString;
use rayon::prelude::*;
use reqwest::Client;

use crate::domain::models::statement::model;
use crate::domain::repositories::statement::{queries, repository};
use crate::infrastructure::jquants_api::JQuantsAPI;
use crate::infrastructure::repositories::statement::structures::jquants_api::Structure;

const STATEMENT_URL: &str = "https://api.jquants.com/v1/fins/statements";

#[async_trait]
impl repository::Statement for JQuantsAPI {
    async fn get_statement<'a>(
        &self,
        query: queries::get_statement::Query<'a>,
    ) -> anyhow::Result<model::Statement> {
        let qs = QueryString::dynamic().with_value("code", query.code);
        let url = format!("{STATEMENT_URL}{qs}");
        let id_token = self.id_token.as_str();
        let response = Client::new().get(url).bearer_auth(id_token).send().await?;
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
                TryFrom::try_from(Structure {
                    code: query.code.to_string(),
                    value,
                })
                .ok()
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
    use chrono::NaiveDate;
    use pretty_assertions::assert_eq;
    use rstest::*;

    use crate::domain::models::statement::model;
    use crate::domain::repositories::statement::queries;
    use crate::domain::repositories::statement::repository::Statement;
    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use crate::shared::jquants_api::setup::Setup;

    #[fixture]
    async fn setup() -> Result<Token> {
        Setup::run().await
    }

    #[rstest]
    #[tokio::test]
    async fn get_statement_test(#[future] setup: Result<Token>) -> Result<()> {
        let token = setup.await?;
        let repository = JQuantsAPI::new(token.id_token.value)?;
        let query = queries::get_statement::Query { code: "8473" };
        let actual = repository.get_statement(query).await?;
        let expected = model::Statement {
            code: "8473".to_string(),
            disclosed_date: NaiveDate::from_ymd_opt(2025, 5, 9).unwrap(),
            eps: 536.09,
            bps: 4162.73,
            net_sales: 1443733000000,
            opp: 0,
            orp: 0,
            profit: 162120000000,
            equity: 1763793000000,
            total_assets: 32113430000000,
        };
        assert_eq!(actual, expected);

        let query = queries::get_statement::Query { code: "????" };
        let actual = repository
            .get_statement(query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[statements] in response not found. code: ????";
        assert_eq!(actual, expected);

        let query = queries::get_statement::Query { code: "2995" };
        let actual = repository
            .get_statement(query)
            .await
            .unwrap_err()
            .to_string();
        let expected = "response[statements] in response is empty array. code: 2995";
        assert_eq!(actual, expected);

        Ok(())
    }
}
