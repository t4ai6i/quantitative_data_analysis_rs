use serde_json::Value;

use crate::domain::models::company::model;
use crate::shared::from_json_string_value::FromJsonStringValue;

pub struct Response<'a>(pub &'a Value);

impl From<Response<'_>> for model::RowCompany {
    fn from(value: Response<'_>) -> Self {
        let Response(value) = value;
        let code = String::from_json_string_value("Code", value).ok();
        let name = String::from_json_string_value("CompanyNameEnglish", value).ok();
        let market = String::from_json_string_value("MarketCode", value).ok();
        Self {
            code,
            name,
            market,
            symbol: Some("".to_string()),
        }
    }
}
