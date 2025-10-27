use anyhow::{Error, Result};
use deref_derive::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Deref, DerefMut)]
pub struct Symbol(String);

impl TryFrom<(&str, &str)> for Symbol {
    type Error = Error;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let (code, market) = value;
        let market = match market {
            // 暫定で、Yahoo Finance API からのレスポンスは "T" に統一
            "JPX" | "T" | "東証" | "東S" | "東P" | "東G" | "名N" => "T".to_string(),
            _ => market.to_string(),
        };
        let symbol = if market.is_empty() {
            code.to_owned()
        } else {
            format!("{}.{}", code, &market)
        };
        Ok(Self(symbol))
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::infrastructure::symbol::Symbol;

    #[test]
    fn symbol_test() -> Result<()> {
        let code = "84730";
        let market = "0110";
        let actual = Symbol::try_from((code, market))?;
        assert_eq!(actual.as_str(), "84730.0110");
        Ok(())
    }

    #[test]
    fn symbol_for_yahoo_finance_api_test() -> Result<()> {
        let code = "8473";
        let market = "東証";
        let actual = Symbol::try_from((code, market))?;
        assert_eq!(actual.as_str(), "8473.T");
        Ok(())
    }
}
