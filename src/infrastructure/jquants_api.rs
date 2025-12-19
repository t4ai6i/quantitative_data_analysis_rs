use anyhow::{ensure, Context, Result};
use bytestring::ByteString;
use chrono::{Days, NaiveDateTime, Utc};
use chrono_tz::Asia::Tokyo;
use itertools::Either;
use query_string_builder::QueryString;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const AUTH_USER_URL: &str = "https://api.jquants.com/v1/token/auth_user";
const AUTH_REFRESH_URL: &str = "https://api.jquants.com/v1/token/auth_refresh";

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RefreshToken {
    pub value: String,
    pub expires_in: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IdToken {
    pub value: String,
    pub expires_in: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Token {
    pub refresh_token: RefreshToken,
    pub id_token: IdToken,
}

impl Token {
    pub async fn update_token(
        token: Result<Vec<u8>>,
        mailaddress: impl Into<String>,
        password: impl Into<String>,
        now: NaiveDateTime,
    ) -> Result<Either<Token, Token>> {
        let token = match token {
            Err(_) => {
                let refresh_token = JQuantsAPI::get_refresh_token(mailaddress, password).await?;
                let id_token = JQuantsAPI::get_id_token(&refresh_token).await?;
                let token = Token {
                    refresh_token,
                    id_token,
                };
                Either::Right(token)
            }
            Ok(body) => {
                let token: Token = serde_json::from_slice(&body)?;
                let is_refresh_token_expired = token.is_refresh_token_expired(now);
                let is_id_token_expired = token.is_id_token_expired(now);
                match (is_refresh_token_expired, is_id_token_expired) {
                    (true, _) => {
                        let refresh_token =
                            JQuantsAPI::get_refresh_token(mailaddress, password).await?;
                        let id_token = JQuantsAPI::get_id_token(&refresh_token).await?;
                        let token = Token {
                            refresh_token,
                            id_token,
                        };
                        Either::Right(token)
                    }
                    (_, true) => {
                        let id_token = JQuantsAPI::get_id_token(&token.refresh_token).await?;
                        let token = Token { id_token, ..token };
                        Either::Right(token)
                    }
                    _ => Either::Left(token),
                }
            }
        };
        Ok(token)
    }

    fn is_refresh_token_expired(&self, now: NaiveDateTime) -> bool {
        self.refresh_token.expires_in < now
    }

    fn is_id_token_expired(&self, now: NaiveDateTime) -> bool {
        self.id_token.expires_in < now
    }
}

#[derive(Deserialize)]
struct AuthUserResponse {
    #[serde(rename = "refreshToken")]
    refresh_token: String,
}

#[derive(Deserialize)]
struct AuthRefreshResponse {
    #[serde(rename = "idToken")]
    id_token: String,
}

/// Parses a `/token/auth_user` response body and returns the contained refresh token string.
/// The function validates the JSON structure and ensures the token is not empty.
pub fn parse_refresh_token_payload(body: &[u8]) -> Result<String> {
    let parsed: AuthUserResponse =
        serde_json::from_slice(body).context("Failed to deserialize refresh token response")?;
    let trim = parsed.refresh_token.trim();
    ensure!(!trim.is_empty(), "refreshToken field is empty");
    Ok(trim.to_string())
}

/// Parses a `/token/auth_refresh` response body and returns the contained ID token string.
/// The function validates the JSON structure and ensures the token is not empty.
pub fn parse_id_token_payload(body: &[u8]) -> Result<String> {
    let parsed: AuthRefreshResponse =
        serde_json::from_slice(body).context("Failed to deserialize id token response")?;
    let trim = parsed.id_token.trim();
    ensure!(!trim.is_empty(), "idToken field is empty");
    Ok(trim.to_string())
}

pub struct JQuantsAPI {
    pub id_token: ByteString,
}

impl JQuantsAPI {
    pub fn new(id_token: ByteString) -> Result<Self> {
        Ok(Self { id_token })
    }

    pub async fn get_refresh_token(
        mail_address: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<RefreshToken> {
        let mut params = HashMap::new();
        params.insert("mailaddress", mail_address.into());
        params.insert("password", password.into());
        let response = Client::new()
            .post(AUTH_USER_URL)
            .json(&params)
            .send()
            .await?;
        let body = response.bytes().await?;
        let refresh_token = parse_refresh_token_payload(&body)?;
        let now = Utc::now().with_timezone(&Tokyo).naive_local();
        Ok(RefreshToken {
            value: refresh_token,
            expires_in: now.checked_add_days(Days::new(7)).unwrap(),
        })
    }

    pub async fn get_id_token(refresh_token: &RefreshToken) -> Result<IdToken> {
        let qs = QueryString::dynamic().with_value("refreshtoken", &refresh_token.value);
        let auth_refresh_url = format!("{AUTH_REFRESH_URL}{qs}");
        let response = Client::new().post(auth_refresh_url).send().await?;
        let body = response.bytes().await?;
        let id_token = parse_id_token_payload(&body)?;
        let now = Utc::now().with_timezone(&Tokyo).naive_local();
        Ok(IdToken {
            value: id_token,
            expires_in: now.checked_add_days(Days::new(1)).unwrap(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_id_token_payload, parse_refresh_token_payload};
    use anyhow::Context;
    use chrono::Utc;
    use chrono_tz::Asia::Tokyo;

    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};

    #[tokio::test]
    async fn jquants_api_test() -> anyhow::Result<()> {
        let mailaddress = std::env::var("JQUANTS_MAIL_ADDRESS")
            .with_context(|| "E-mail address must be set up for JQUANTS".to_string())?;
        let password = std::env::var("JQUANTS_PASSWORD")
            .with_context(|| "Password must be set up for JQUANTS".to_string())?;
        let refresh_token = JQuantsAPI::get_refresh_token(mailaddress, password).await?;
        assert!(!&refresh_token.value.is_empty());

        let id_token = JQuantsAPI::get_id_token(&refresh_token).await?;
        assert!(!&id_token.value.is_empty());

        let token = Token {
            refresh_token,
            id_token,
        };
        let now = Utc::now().with_timezone(&Tokyo).naive_local();
        assert!(!token.is_refresh_token_expired(now));
        assert!(!token.is_id_token_expired(now));
        Ok(())
    }

    #[test]
    fn parse_refresh_token_payload_empty_fails() {
        let body = br#"{"refreshToken": ""}"#;
        assert!(parse_refresh_token_payload(body).is_err());
    }

    #[test]
    fn parse_refresh_token_payload_missing_field_fails() {
        let body = br#"{"unexpected": "value"}"#;
        assert!(parse_refresh_token_payload(body).is_err());
    }

    #[test]
    fn parse_id_token_payload_empty_fails() {
        let body = br#"{"idToken": "  "}"#;
        assert!(parse_id_token_payload(body).is_err());
    }

    #[test]
    fn parse_id_token_payload_missing_field_fails() {
        let body = br#"{"error": "invalid"}"#;
        assert!(parse_id_token_payload(body).is_err());
    }

    #[test]
    fn parse_refresh_token_payload_handles_valid_body() {
        let body = br#"{"refreshToken": "token123"}"#;
        let parsed = parse_refresh_token_payload(body).expect("valid payload should parse");
        assert_eq!(parsed, "token123");
    }

    #[test]
    fn parse_refresh_token_payload_rejects_malformed_json() {
        let body = br#"{"refresh": "token123"}"#;
        assert!(parse_refresh_token_payload(body).is_err());
    }

    #[test]
    fn parse_id_token_payload_handles_valid_body() {
        let body = br#"{"idToken": "id-abc"}"#;
        let parsed = parse_id_token_payload(body).expect("valid payload should parse");
        assert_eq!(parsed, "id-abc");
    }

    #[test]
    fn parse_id_token_payload_rejects_empty_token() {
        let body = br#"{"idToken": ""}"#;
        assert!(parse_id_token_payload(body).is_err());
    }
}
