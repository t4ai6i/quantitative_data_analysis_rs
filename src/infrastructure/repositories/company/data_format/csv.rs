use crate::domain::models::company::model::Company;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use crate::infrastructure::symbol::Symbol;
use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Csv {
    #[serde(rename = "コード")]
    pub code: String,
    #[serde(rename = "銘柄名")]
    pub name: String,
    #[serde(rename = "市場")]
    pub market: String,
    #[serde(rename = "現在値")]
    pub value: String,
    #[serde(rename = "前日比(%)")]
    pub day_before: String,
}

impl TryFrom<Csv> for Company {
    type Error = Error;

    fn try_from(value: Csv) -> Result<Self, Self::Error> {
        let Csv {
            code, name, market, ..
        } = value;
        let mut symbol = Symbol::try_from((code.as_str(), market.as_str()))?;
        Ok(Self {
            code,
            name,
            market,
            symbol: std::mem::take(&mut symbol),
        })
    }
}

impl FromSlice for Csv {
    type Deserialize = Csv;
    type Item = Company;

    fn data_format() -> DataFormat {
        DataFormat::CSV
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::from_slice::FromSlice;
    const COMPANIES: &[u8] = include_bytes!("../../../../../assets/companies.csv");

    #[test]
    fn vec_company_test() {
        let vec_company = Csv::from_slice::<true>(COMPANIES);
        assert_eq!(vec_company.len(), 61);
    }
}
