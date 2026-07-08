use deref_derive::{Deref, DerefMut};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct RowCompany {
    pub code: Option<String>,
    pub name: Option<String>,
    pub market: Option<String>,
    pub product_category: Option<String>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Company {
    pub code: String,
    pub name: String,
    pub market: String,
    pub product_category: Option<String>,
    pub symbol: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct Companies(pub Vec<Company>);

impl TryFrom<RowCompany> for Company {
    type Error = anyhow::Error;

    fn try_from(value: RowCompany) -> Result<Self, Self::Error> {
        let RowCompany {
            code,
            name,
            market,
            product_category,
            symbol,
        } = value;
        Ok(Self {
            code: code.ok_or_else(|| anyhow::anyhow!("code is None"))?,
            name: name.ok_or_else(|| anyhow::anyhow!("name is None"))?,
            market: market.ok_or_else(|| anyhow::anyhow!("market is None"))?,
            product_category,
            symbol: symbol.ok_or_else(|| anyhow::anyhow!("symbol is None"))?,
        })
    }
}
