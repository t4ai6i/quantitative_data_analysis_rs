use anyhow::{Context, Result};
use bytestring::ByteString;

pub struct Setup {}

impl Setup {
    pub async fn run() -> Result<ByteString> {
        let api_key = std::env::var("JQUANTS_API_KEY")
            .with_context(|| "JQUANTS_API_KEY MUST be set up for JQUANTS".to_string())?;
        Ok(ByteString::from(api_key))
    }
}
