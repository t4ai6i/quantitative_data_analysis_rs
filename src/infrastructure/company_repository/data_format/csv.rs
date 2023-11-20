use crate::domain::entity::company::Company;
use crate::infrastructure::csv_ext::CsvExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct CompanyCsvRow {
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

impl From<CompanyCsvRow> for Company {
    fn from(value: CompanyCsvRow) -> Self {
        let CompanyCsvRow {
            code, name, market, ..
        } = value;
        Self { code, name, market }
    }
}

impl CsvExt for CompanyCsvRow {
    type CSVFormat = CompanyCsvRow;
    type Item = Company;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::csv_ext::CsvExt;
    const COMPANIES: &[u8] = include_bytes!("../../../../assets/companies.csv");

    #[test]
    fn vec_company_test() {
        let vec_company = CompanyCsvRow::from_slice::<true>(COMPANIES);
        assert_eq!(vec_company.len(), 61);
    }
}
