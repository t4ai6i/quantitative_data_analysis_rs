use crate::domain::models::company::model;
use anyhow::{Context, Error};
use serde_json::Value;

pub struct Response<'a>(pub &'a Value);

impl TryFrom<Response<'_>> for model::Company {
    type Error = Error;

    fn try_from(value: Response<'_>) -> Result<Self, Self::Error> {
        let Response(value) = value;
        let code = value["Code"]
            .as_str()
            .with_context(|| "[Code] not found".to_string())?;
        let company_name_english = value["CompanyNameEnglish"]
            .as_str()
            .with_context(|| "[CompanyNameEnglish] not found".to_string())?;
        let market_code = value["MarketCode"]
            .as_str()
            .with_context(|| "[MarketCode] not found".to_string())?;
        Ok(Self {
            code: code.to_string(),
            name: company_name_english.to_string(),
            market: market_code.to_string(),
            symbol: "".to_string(),
        })
    }
}
