use anyhow::{Context, Result};
use bytestring::ByteString;
use chrono::Utc;
use chrono_tz::Asia::Tokyo;
use dirs::config_dir;
use itertools::Either;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::fs::read;

use crate::infrastructure::jquants_api::Token;

const TOKEN_FILE_NAME: &str = "jquants_api_token.json";
const APP_CONFIG_DIR: &str = "quantitative-data-analysis-rs";

pub struct Setup {}

impl Setup {
    pub async fn run() -> Result<ByteString> {
        let token_path = resolve_token_path()?;
        let binary: Result<Vec<u8>> = read(&token_path)
            .await
            .with_context(|| format!("Token file not found: {}", token_path.display()));
        let mailaddress = std::env::var("JQUANTS_MAIL_ADDRESS")
            .with_context(|| "E-mail address MUST be set up for JQUANTS".to_string())?;
        let password = std::env::var("JQUANTS_PASSWORD")
            .with_context(|| "Password MUST be set up for JQUANTS".to_string())?;
        let now = Utc::now().with_timezone(&Tokyo).naive_local();

        let update_token = Token::update_token(binary, &mailaddress, &password, now).await?;

        let token = match update_token {
            Either::Right(token) => {
                let serialized = serde_json::to_vec(&token)?;
                write_secure_token(&token_path, &serialized).await?;
                token
            }
            Either::Left(token) => token,
        };

        let token = ByteString::from(token.id_token.value);
        Ok(token)
    }
}

fn resolve_token_path() -> Result<PathBuf> {
    let mut config_root =
        config_dir().ok_or_else(|| anyhow::anyhow!("Unable to determine config directory"))?;
    config_root.push(APP_CONFIG_DIR);
    ensure_private_dir(&config_root)?;
    Ok(config_root.join(TOKEN_FILE_NAME))
}

fn ensure_private_dir(dir: &Path) -> Result<()> {
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

async fn write_secure_token(path: &Path, data: &[u8]) -> Result<()> {
    tokio::fs::write(path, data)
        .await
        .with_context(|| format!("Failed to write token file: {}", path.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        tokio::fs::set_permissions(path, perms).await?;
    }
    Ok(())
}
