use anyhow::Error;
use serde::{Deserialize, Serialize};

use crate::domain::models::company::model;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use crate::infrastructure::symbol::Symbol;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Structure {
    pub code: String,
    pub name: String,
    pub market: String,
}

impl TryFrom<Structure> for model::Company {
    type Error = Error;

    fn try_from(value: Structure) -> Result<Self, Self::Error> {
        let Structure { code, name, market } = value;
        let mut symbol = Symbol::try_from((code.as_str(), market.as_str()))?;
        Ok(Self {
            code,
            name,
            market,
            symbol: std::mem::take(&mut symbol),
        })
    }
}

impl FromSlice for Structure {
    type Deserialize = Structure;
    type Item = model::Company;

    fn data_format() -> DataFormat {
        DataFormat::Tsv
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::company::structures::internal::tsv::Structure;

    const COMPANIES: &[u8] = include_bytes!("../../../../../../assets/companies.tsv");

    #[test]
    fn vec_company_test() {
        let vec_company = Structure::from_slice::<false>(COMPANIES);
        assert_eq!(vec_company.len(), 4412);
    }
}
