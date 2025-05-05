use crate::infrastructure::jquants_api::Token;
use anyhow::Context;
use chrono::Utc;
use chrono_tz::Asia::Tokyo;
use itertools::Either;
use tokio::fs::{read, write};

const TOKEN_FILE_NAME_PREFIX: &str = "jquants_api_token";
const TOKEN_FILE_NAME_SUFFIX: &str = ".json";

pub struct Setup {}

impl Setup {
    pub async fn run() -> anyhow::Result<Token> {
        let mailaddress = std::env::var("JQUANTS_MAIL_ADDRESS")
            .with_context(|| "E-mail address MUST be set up for JQUANTS".to_string())?;
        let password = std::env::var("JQUANTS_PASSWORD")
            .with_context(|| "Password MUST be set up for JQUANTS".to_string())?;
        let now = Utc::now().with_timezone(&Tokyo).naive_local();
        let token_file_name = format!("{}{}", TOKEN_FILE_NAME_PREFIX, TOKEN_FILE_NAME_SUFFIX);
        let binary = read(&token_file_name).await;
        let update_token = Token::update_token(binary, &mailaddress, &password, now).await?;
        let token = match update_token {
            Either::Right(token) => {
                let serialized = serde_json::to_vec(&token)?;
                write(&token_file_name, &serialized).await?;
                token
            }
            Either::Left(token) => token,
        };
        Ok(token)
    }
}
