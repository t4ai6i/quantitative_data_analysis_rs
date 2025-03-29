use anyhow::Context;
use csv::{ReaderBuilder, Writer};
use indoc::indoc;
use quantitative_data_analysis_rs::infrastructure::stock_repository::data_format::csv::Csv;
use quantitative_data_analysis_rs::presenter::view_model::macos_analysis::MACOSAnalysis;

const CSV_8473: &[u8] = include_bytes!("../assets/8473.T.csv");

#[test]
fn csv_sandbox() {
    let mut reader = ReaderBuilder::new().has_headers(true).from_reader(CSV_8473);
    assert_eq!(
        reader.headers().unwrap(),
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
    let record = record.context("cannot read first line.").unwrap();
    let actual: Csv = record.unwrap();
    let expected = indoc! {r#"{
        "Date":"2022-09-09",
        "Open":2662.0,
        "High":2695.0,
        "Low":2662.0,
        "Close":2685.0,
        "Adj Close":2535.687744,
        "Volume":1482200
    }"#};
    let expected: Csv = serde_json::from_str(expected).unwrap();
    assert_eq!(actual, expected);

    let json = indoc! {r#"{
            "pattern":"GoldenCross",
            "latest_chance":"2023-08-30",
            "rate_of_chance":27.27272727272727
        }"#};
    let macos_analysis: MACOSAnalysis = serde_json::from_str(json).unwrap();
    let mut builder = Writer::from_writer(vec![]);
    let _ = builder.serialize(macos_analysis);
    let data = String::from_utf8(builder.into_inner().unwrap()).unwrap();
    assert_eq!(
        data,
        "pattern,latest_chance,rate_of_chance\nGoldenCross,2023-08-30,27.27272727272727\n"
    );
}
