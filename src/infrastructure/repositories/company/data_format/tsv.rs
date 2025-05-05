use crate::domain::models::company::model::Company;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use crate::infrastructure::symbol::Symbol;
use anyhow::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Tsv {
    pub code: String,
    pub name: String,
}

impl TryFrom<Tsv> for Company {
    type Error = Error;

    fn try_from(value: Tsv) -> Result<Self, Self::Error> {
        let Tsv { code, name } = value;
        let market = "T".to_string();
        let mut symbol = Symbol::try_from((code.as_str(), market.as_str()))?;
        Ok(Self {
            code,
            name,
            market,
            symbol: std::mem::take(&mut symbol),
        })
    }
}

impl FromSlice for Tsv {
    type Deserialize = Tsv;
    type Item = Company;

    fn data_format() -> DataFormat {
        DataFormat::TSV
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::from_slice::FromSlice;
    const COMPANIES: &[u8] = include_bytes!("../../../../../assets/companies.tsv");

    #[test]
    fn vec_company_test() {
        let vec_company = Tsv::from_slice::<false>(COMPANIES);
        assert_eq!(vec_company.len(), 5);
    }
}
