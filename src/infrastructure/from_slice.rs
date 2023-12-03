use csv::ReaderBuilder;
use itertools::Itertools;
use serde::de::DeserializeOwned;

pub enum DataFormat {
    CSV,
    TSV,
}

pub trait FromSlice {
    type Deserialize: DeserializeOwned;
    type Item: From<Self::Deserialize>;

    fn data_format() -> DataFormat;

    fn from_slice<const B: bool>(value: &[u8]) -> Vec<Self::Item> {
        let mut reader = match Self::data_format() {
            DataFormat::CSV => {
                if B {
                    ReaderBuilder::new().has_headers(true).from_reader(value)
                } else {
                    ReaderBuilder::new().has_headers(false).from_reader(value)
                }
            }
            DataFormat::TSV => {
                if B {
                    ReaderBuilder::new()
                        .delimiter(b'\t')
                        .has_headers(true)
                        .from_reader(value)
                } else {
                    ReaderBuilder::new()
                        .delimiter(b'\t')
                        .has_headers(false)
                        .from_reader(value)
                }
            }
        };
        reader
            .deserialize::<Self::Deserialize>()
            .filter_map(Result::ok)
            .map(Self::Item::from)
            .collect_vec()
    }
}
