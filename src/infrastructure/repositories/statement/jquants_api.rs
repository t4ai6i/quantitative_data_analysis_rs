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
            bail!("[statements] in response not found");
        };
        // 次の項目が存在するものを選択。FiscalYear,BookValuePerShare,EarningsPerShare
        // 上記の条件を満たしているもので最新を選択。API Docに以下の記述があるため、日付でのソートは行っていない。
        // 「DisclosureNumber: APIから出力されるjsonは開示番号で昇順に並んでいます。」
        response
            .par_iter()
            .filter_map(|value| {
                TryFrom::try_from(Structure {
                    code: query.code.to_string(),
                    value,
                })
                .ok()
            })
            .reduce_with(|_a, b| b)
            .with_context(|| {
                format!(
                    "struct model::Statement couldn't construct. code: {}",
                    query.code
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rstest::*;

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
        let actual = repository.get_statement(query).await;
        assert!(actual.is_ok());
        Ok(())
    }
}
