use anyhow::{bail, Error, Result};
use deref_derive::{Deref, DerefMut};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Deref, DerefMut)]
pub struct Symbol(String);

impl TryFrom<(&str, &str)> for Symbol {
    type Error = Error;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let (code, market) = value;
        let market = match market {
            "JPX" | "T" | "東証" | "東S" | "東P" | "東G" | "名N" => "T".to_string(),
            "" => "".to_string(),
            _ => bail!("Unknown market: {}", market),
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
    use crate::infrastructure::symbol::Symbol;

    #[test]
    fn symbol_test() -> anyhow::Result<()> {
        let code = "8473";
        let market = "東証";
        let actual = Symbol::try_from((code, market))?;
        assert_eq!(actual.as_str(), "8473.T");
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Unknown market: hogehoge")]
    fn symbol_invalid_market_test() {
        let code = "8473";
        let market = "hogehoge";
        let _ = Symbol::try_from((code, market)).unwrap();
    }
}
