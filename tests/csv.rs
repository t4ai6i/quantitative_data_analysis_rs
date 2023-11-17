use anyhow::{Context, Result};
use csv::ReaderBuilder;
use indoc::indoc;
use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::CSVFormat;

const CSV_8473: &[u8] = include_bytes!("../assets/8473.T.csv");

#[test]
fn csv_sandbox() -> Result<()> {
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(CSV_8473);
    assert_eq!(
        reader.headers()?,
        vec![
            "Date",
            "Open",
            "High",
            "Low",
            "Close",
            "Adj Close",
            "Volume"
        ]
    );
    let record = reader.deserialize().next();
    let record = record.context("cannot read first line.")?;
    let actual: CSVFormat = record?;
    let expected = indoc! {r#"{
        "Date":"2022-09-09",
        "Open":2662.0,
        "High":2695.0,
        "Low":2662.0,
        "Close":2685.0,
        "Adj Close":2535.687744,
        "Volume":1482200
    }"#};
    let expected: CSVFormat = serde_json::from_str(expected)?;
    assert_eq!(actual, expected);
    Ok(())
}
