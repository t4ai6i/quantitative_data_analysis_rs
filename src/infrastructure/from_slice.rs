use anyhow::bail;
use csv::ReaderBuilder;
use itertools::Itertools;
use serde::de::DeserializeOwned;
use std::backtrace::Backtrace;
use std::fmt::{Debug, Display};

pub enum DataFormat {
    CSV,
    TSV,
}

type DeserializationResult<T, E> = Vec<Result<T, E>>;

pub trait FromSlice {
    type Deserialize: DeserializeOwned + Debug;
    type Item: TryFrom<Self::Deserialize> + Debug;

    fn data_format() -> DataFormat;

    fn from_slice<const B: bool>(value: &[u8]) -> Vec<csv::Result<Self::Deserialize>> {
        let mut reader = match Self::data_format() {
            DataFormat::CSV => ReaderBuilder::new().has_headers(B).from_reader(value),
            DataFormat::TSV => ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(B)
                .from_reader(value),
        };
        reader.deserialize::<Self::Deserialize>().collect_vec()
    }

    fn from_deserialize(
        ds: Vec<Self::Deserialize>,
    ) -> DeserializationResult<Self::Item, <Self::Item as TryFrom<Self::Deserialize>>::Error> {
        ds.into_iter().map(Self::Item::try_from).collect_vec()
    }

    fn process_tabular_data(file: &[u8], has_headers: bool) -> anyhow::Result<Vec<Self::Item>>
    where
        <Self::Item as TryFrom<Self::Deserialize>>::Error: Debug + Display,
    {
        let from_slice = if has_headers {
            Self::from_slice::<true>(file)
        } else {
            Self::from_slice::<false>(file)
        };
        let (successes, failures): (Vec<_>, Vec<_>) =
            from_slice.into_iter().partition(Result::is_ok);
        if !failures.is_empty() {
            let failures = failures
                .into_iter()
                .map(|e| e.unwrap_err().to_string())
                .join("\n");
            bail!(
                "Error from_slice: {}\n{}",
                failures,
                Backtrace::force_capture()
            );
        }
        let successes: Vec<_> = successes.into_iter().map(|e| e.unwrap()).collect();
        let (successes, failures): (Vec<_>, Vec<_>) = Self::from_deserialize(successes)
            .into_iter()
            .partition(Result::is_ok);
        if !failures.is_empty() {
            let failures = failures
                .into_iter()
                .map(|e| e.unwrap_err().to_string())
                .join("\n");
            bail!(
                "Error from_deserialize: {}\n{}",
                failures,
                Backtrace::force_capture()
            );
        }
        let successes = successes
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<Self::Item>>();
        Ok(successes)
    }
}
