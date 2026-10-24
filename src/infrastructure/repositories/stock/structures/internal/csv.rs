use crate::domain::models::stock::model;
use crate::infrastructure::from_slice::{DataFormat, FromSlice};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Structure {
    #[serde(rename = "Date")]
    pub date: NaiveDate,
    #[serde(rename = "Open")]
    pub open: f64,
    #[serde(rename = "High")]
    pub high: f64,
    #[serde(rename = "Low")]
    pub low: f64,
    #[serde(rename = "Close")]
    pub close: f64,
    #[serde(rename = "Adj Open")]
    pub adj_open: f64,
    #[serde(rename = "Adj High")]
    pub adj_high: f64,
    #[serde(rename = "Adj Low")]
    pub adj_low: f64,
    #[serde(rename = "Adj Close")]
    pub adj_close: f64,
    #[serde(rename = "Volume")]
    pub volume: u64,
    #[serde(rename = "Adj Volume")]
    pub adj_volume: f64,
}

impl TryFrom<Structure> for model::RowStock {
    type Error = anyhow::Error;

    fn try_from(value: Structure) -> Result<Self, Self::Error> {
        let Structure {
            date,
            open,
            high,
            low,
            close,
            adj_open,
            adj_high,
            adj_low,
            adj_close,
            volume,
            adj_volume,
        } = value;
        let row = Self {
            date: Some(date),
            open: Some(open),
            high: Some(high),
            low: Some(low),
            close: Some(close),
            adj_open: Some(adj_open),
            adj_high: Some(adj_high),
            adj_low: Some(adj_low),
            adj_close: Some(adj_close),
            volume: Some(volume),
            adj_volume: Some(adj_volume),
        };
        model::Stock::try_from(row)?;
        Ok(row)
    }
}

impl FromSlice for Structure {
    type Deserialize = Structure;
    type Item = model::RowStock;

    fn data_format() -> DataFormat {
        DataFormat::Csv
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::from_slice::FromSlice;
    use crate::infrastructure::repositories::stock::structures::internal::csv::Structure;

    const CSV_8473: &[u8] = include_bytes!("../../../../../../assets/8473.T.csv");

    #[test]
    fn vec_stock_test() {
        let vec_stock = Structure::process_tabular_data(CSV_8473, true).unwrap();
        assert_eq!(vec_stock.len(), 246);
    }

    #[test]
    fn rejects_legacy_csv() {
        let csv = b"Date,Open,High,Low,Close,Adj Close,Volume\n2024-01-01,10,12,9,11,11,100\n";
        assert!(Structure::process_tabular_data(csv, true).is_err());
    }

    #[test]
    fn validates_adjusted_fields() {
        let header =
            "Date,Open,High,Low,Close,Adj Open,Adj High,Adj Low,Adj Close,Volume,Adj Volume\n";
        for invalid in ["", "NaN", "inf", "-1"] {
            let csv = format!("{header}2024-01-01,10,12,9,11,5,6,4.5,5.5,100,{invalid}\n");
            assert!(Structure::process_tabular_data(csv.as_bytes(), true).is_err());
        }
        let csv = format!("{header}2024-01-01,10,12,9,11,5,6,4.5,5.5,100,200.5\n");
        let rows = Structure::process_tabular_data(csv.as_bytes(), true).unwrap();
        assert_eq!(rows[0].adj_volume, Some(200.5));
        assert_eq!(rows[0].adj_open, Some(5.0));
    }
}
