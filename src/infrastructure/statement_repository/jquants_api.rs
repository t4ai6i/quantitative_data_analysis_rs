use crate::domain::models::statement::model::Statement;
use crate::domain::repositories::statement_repository::StatementRepository;
use crate::infrastructure::jquants_api::JQuantsAPI;
use anyhow::{bail, Context};
use async_trait::async_trait;
use chrono::NaiveDate;
use query_string_builder::QueryString;
use reqwest::Client;
use std::str::FromStr;

const STATEMENT_URL: &str = "https://api.jquants.com/v1/fins/statements";

#[async_trait]
impl StatementRepository for JQuantsAPI {
    async fn get_statement(&self, code: &str) -> anyhow::Result<Statement> {
        let qs = QueryString::dynamic().with_value("code", code);
        let url = format!("{STATEMENT_URL}{qs}");
        let id_token = self.id_token.as_str();
        let response = Client::new().get(url).bearer_auth(id_token).send().await?;
        let response = &response.json::<serde_json::Value>().await?;
        let Some(response) = response["statements"].as_array() else {
            bail!("[statements] in response not found");
        };
        // 次の項目が存在するものを選択。FY,BookValuePerShare,EarningsPerShare
        // 上記の条件を満たしているもので最新を選択。API Docに以下の記述があるため、日付でのソートは行っていない。
        // 「DisclosureNumber: APIから出力されるjsonは開示番号で昇順に並んでいます。」
        response
            .iter()
            .filter_map(|statement| {
                let disclosed_date = statement["DisclosedDate"].as_str();
                let type_of_current_period = statement["TypeOfCurrentPeriod"].as_str();
                let earnings_per_share = statement["EarningsPerShare"].as_str();
                let book_value_per_share = statement["BookValuePerShare"].as_str();
                let disclosed_date = disclosed_date?;
                let disclosed_date = NaiveDate::from_str(disclosed_date).ok()?;
                let type_of_current_period = type_of_current_period?;
                if type_of_current_period.ne("FY") {
                    return None;
                };
                let earnings_per_share = earnings_per_share?;
                let eps = earnings_per_share.parse::<f64>().ok()?;
                let book_value_per_share = book_value_per_share?;
                let bps = book_value_per_share.parse::<f64>().ok()?;
                Some(Statement {
                    code: code.to_string(),
                    disclosed_date,
                    eps,
                    bps,
                })
            })
            .last()
            .with_context(|| format!("struct Statement couldn't construct. code: {}", code))
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::repositories::statement_repository::StatementRepository;
    use crate::infrastructure::data_format::DataFormat;
    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use crate::utils::jquants_api::setup::Setup;
    use anyhow::Result;
    use rstest::*;

    #[fixture]
    async fn setup() -> Result<Token> {
        Setup::run().await
    }

    #[rstest]
    #[tokio::test]
    async fn get_statement_test(#[future] setup: Result<Token>) -> Result<()> {
        let token = setup.await?;
        let code = "8473";
        let data_format = DataFormat::JQuantsAPI;
        let repository = JQuantsAPI::new(&token.id_token.value, data_format)?;
        let actual = repository.get_statement(code).await;
        assert!(actual.is_ok());
        Ok(())
    }
}
