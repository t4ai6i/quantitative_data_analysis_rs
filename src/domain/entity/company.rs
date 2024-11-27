use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Company {
    pub code: String,
    pub name: String,
    pub market: String,
    pub symbol: String,
}

impl Company {
    pub fn symbol(code: &str, market: &str) -> String {
        let market = match market {
            "JPX" | "T" | "東証" | "東S" | "東P" | "東G" | "名N" => "T".to_string(),
            "" => "".to_string(),
            _ => panic!("Unknown market: {}", market),
        };
        if market.is_empty() {
            code.to_string()
        } else {
            format!("{}.{}", code, &market)
        }
    }
}

impl From<&Value> for Company {
    fn from(value: &Value) -> Self {
        Self {
            code: value["Code"].as_str().unwrap().to_string(),
            name: value["CompanyNameEnglish"].as_str().unwrap().to_string(),
            market: value["MarketCode"].as_str().unwrap().to_string(),
            symbol: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::entity::company::Company;

    #[test]
    fn symbol_test() {
        let code = "8473";
        let market = "東証";
        let actual = Company::symbol(code, market);
        assert_eq!(actual, "8473.T");
    }

    #[test]
    #[should_panic]
    fn symbol_invalid_market_test() {
        let code = "8473";
        let market = "hogehoge";
        let _ = Company::symbol(code, market);
    }
}
