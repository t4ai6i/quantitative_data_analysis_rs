use anyhow::{Result, ensure};
use bytestring::ByteString;

pub struct JQuantsAPI {
    pub api_key: ByteString,
}

impl JQuantsAPI {
    pub fn new(api_key: ByteString) -> Result<Self> {
        ensure!(
            !api_key.trim().is_empty(),
            "J-Quants API key must not be empty"
        );
        Ok(Self { api_key })
    }
}

#[cfg(test)]
mod tests {
    use bytestring::ByteString;

    use crate::infrastructure::jquants_api::JQuantsAPI;

    #[test]
    fn new_rejects_empty_api_key() {
        let result = JQuantsAPI::new(ByteString::from("  "));
        assert!(result.is_err());
    }

    #[test]
    fn new_accepts_non_empty_api_key() {
        let result = JQuantsAPI::new(ByteString::from("api-key"));
        assert!(result.is_ok());
    }
}
