use crate::domain::entity::company::Company;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Tsv {
    pub code_name: String,
    pub date: String,
    pub a: String,
    pub b: String,
    pub value: String,
    pub c: String,
    pub d: String,
    pub e: String,
    pub f: String,
    pub g: String,
}

impl From<Tsv> for Company {
    fn from(value: Tsv) -> Self {
        let Tsv { code_name, .. } = value;
        let code_name = code_name.split(' ').collect_vec();
        let code = code_name[0].to_string();
        let name = code_name[1].to_string();
        let market = "T".to_string();
        let symbol = Self::symbol(&code, &market);
        Self::new(code, name, market, symbol)
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
        assert_eq!(vec_company.len(), 24);
    }
}
