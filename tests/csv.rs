use anyhow::{Context, Result};
use csv::{ReaderBuilder, Writer};
use indoc::indoc;
use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
use quantitative_data_analysis_rs::presenter::view_model::cross_analysis::CrossAnalysis;

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
    let actual: Csv = record?;
    let expected = indoc! {r#"{
        "Date":"2022-09-09",
        "Open":2662.0,
        "High":2695.0,
        "Low":2662.0,
        "Close":2685.0,
        "Adj Close":2535.687744,
        "Volume":1482200
    }"#};
    let expected: Csv = serde_json::from_str(expected)?;
    assert_eq!(actual, expected);

    let json = indoc! {r#"{
            "code":"8473",
            "symbol":"8473.T",
            "cross_direction":"golden",
            "latest_chance":"2023-08-30",
            "chance_rate":27.27272727272727
        }"#};
    let cross_analysis: CrossAnalysis = serde_json::from_str(json)?;
    let mut builder = Writer::from_writer(vec![]);
    let _ = builder.serialize(cross_analysis);
    let data = String::from_utf8(builder.into_inner()?)?;
    assert_eq!(data, "code,symbol,cross_direction,latest_chance,chance_rate\n8473,8473.T,golden,2023-08-30,27.27272727272727\n");
    Ok(())
}
