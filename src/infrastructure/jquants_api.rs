use crate::domain::entity::company::Company;
use crate::infrastructure::data_format::DataFormat;
use anyhow::{bail, Result};
use chrono::{Days, NaiveDateTime, Utc};
use chrono_tz::Asia::Tokyo;
use itertools::Either;
use query_string_builder::QueryString;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

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
    pub fn has_refresh_token_expired(&self, now: NaiveDateTime) -> bool {
        if self.refresh_token.expires_in < now {
            return true;
        }
        false
    }

    pub fn has_id_token_expired(&self, now: NaiveDateTime) -> bool {
        if self.id_token.expires_in < now {
            return true;
        }
        false
    }

    pub async fn update_token(
        token: io::Result<Vec<u8>>,
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
                let has_refresh_token_expired = token.has_refresh_token_expired(now);
                let has_id_token_expired = token.has_id_token_expired(now);
                match (has_refresh_token_expired, has_id_token_expired) {
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
}

pub struct JQuantsAPI {
    pub data_format: DataFormat,
    pub id_token: String,
}

impl JQuantsAPI {
    pub fn new(id_token: impl Into<String>, data_format: DataFormat) -> Result<Self> {
        match data_format {
            DataFormat::JQuantsAPI => (),
            _ => {
                bail!(format!(
                    "Unsupported data format: {:?} at {}:{}",
                    data_format,
                    file!(),
                    line!()
                ));
            }
        };
        Ok(Self {
            id_token: Into::into(id_token),
            data_format,
        })
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
        let refresh_token: serde_json::Value = serde_json::from_slice(&body)?;
        let refresh_token = refresh_token["refreshToken"].as_str().unwrap();
        let now = Utc::now().with_timezone(&Tokyo).naive_local();
        Ok(RefreshToken {
            value: refresh_token.to_string(),
            expires_in: now.checked_add_days(Days::new(7)).unwrap(),
        })
    }

    pub async fn get_id_token(refresh_token: &RefreshToken) -> Result<IdToken> {
        let qs = QueryString::dynamic().with_value("refreshtoken", &refresh_token.value);
        let auth_refresh_url = format!("{AUTH_REFRESH_URL}{qs}");
        let response = Client::new().post(auth_refresh_url).send().await?;
        let body = response.bytes().await?;
        let id_token: serde_json::Value = serde_json::from_slice(&body)?;
        let id_token = id_token["idToken"].as_str().unwrap();
        let now = Utc::now().with_timezone(&Tokyo).naive_local();
        Ok(IdToken {
            value: id_token.to_string(),
            expires_in: now.checked_add_days(Days::new(1)).unwrap(),
        })
    }

    pub fn company_from_value(value: &serde_json::Value) -> Company {
        Company {
            code: value["Code"].as_str().unwrap().to_string(),
            name: value["CompanyNameEnglish"].as_str().unwrap().to_string(),
            market: value["MarketCode"].as_str().unwrap().to_string(),
            symbol: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::jquants_api::{JQuantsAPI, Token};
    use anyhow::Context;
    use chrono::Utc;
    use chrono_tz::Asia::Tokyo;

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
        assert!(!token.has_refresh_token_expired(now));
        assert!(!token.has_id_token_expired(now));
        Ok(())
    }
}
