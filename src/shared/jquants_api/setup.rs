use anyhow::{Context, Result};
use chrono::Utc;
use chrono_tz::Asia::Tokyo;
use itertools::Either;
use tokio::fs::{read, write};

use crate::infrastructure::jquants_api::Token;

const TOKEN_FILE_NAME: &str = "jquants_api_token.json";

pub struct Setup {}

impl Setup {
    pub async fn run() -> Result<Token> {
        let binary: Result<Vec<u8>> = read(TOKEN_FILE_NAME)
            .await
            .with_context(|| format!("Token file not found: {}", TOKEN_FILE_NAME));
        let mailaddress = std::env::var("JQUANTS_MAIL_ADDRESS")
            .with_context(|| "E-mail address MUST be set up for JQUANTS".to_string())?;
        let password = std::env::var("JQUANTS_PASSWORD")
            .with_context(|| "Password MUST be set up for JQUANTS".to_string())?;
        let now = Utc::now().with_timezone(&Tokyo).naive_local();

        let update_token = Token::update_token(binary, &mailaddress, &password, now).await?;

        let token = match update_token {
            Either::Right(token) => {
                let serialized = serde_json::to_vec(&token)?;
                write(TOKEN_FILE_NAME, &serialized).await?;
                token
            }
            Either::Left(token) => token,
        };

        Ok(token)
    }
}
