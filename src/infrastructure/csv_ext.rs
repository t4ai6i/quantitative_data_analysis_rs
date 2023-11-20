use csv::ReaderBuilder;
use itertools::Itertools;
use serde::de::DeserializeOwned;

pub trait CsvExt {
    type CSVFormat: DeserializeOwned;
    type Item: From<Self::CSVFormat>;

    fn from_slice<const B: bool>(value: &[u8]) -> Vec<Self::Item> {
        let mut reader = if B {
            ReaderBuilder::new().has_headers(true).from_reader(value)
        } else {
            ReaderBuilder::new().has_headers(false).from_reader(value)
        };
        reader
            .deserialize::<Self::CSVFormat>()
            .filter_map(Result::ok)
            .map(Self::Item::from)
            .collect_vec()
    }
}
