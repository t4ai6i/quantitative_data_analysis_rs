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

impl From<Structure> for model::RowCompany {
    fn from(value: Structure) -> Self {
        let Structure { code, name, market } = value;
        let symbol = Symbol::try_from((code.as_str(), market.as_str()))
            .ok()
            .map(|symbol| symbol.to_string());
        Self {
            code: Some(code),
            name: Some(name),
            market: Some(market),
            product_category: None,
            symbol,
        }
    }
}

impl FromSlice for Structure {
    type Deserialize = Structure;
    type Item = model::RowCompany;

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
