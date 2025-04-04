use crate::domain::models::company::model::Company;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Tsv {
    pub code: String,
    pub name: String,
}

impl From<Tsv> for Company {
    fn from(value: Tsv) -> Self {
        let Tsv { code, name } = value;
        let market = "T".to_string();
        let symbol = Self::symbol(&code, &market);
        Self {
            code,
            name,
            market,
            symbol,
        }
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
    const COMPANIES: &[u8] = include_bytes!("../../../../assets/companies.tsv");

    #[test]
    fn vec_company_test() {
        let vec_company = Tsv::from_slice::<false>(COMPANIES);
        assert_eq!(vec_company.len(), 5);
    }
}
