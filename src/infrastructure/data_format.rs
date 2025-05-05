use std::path::PathBuf;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum DataFormat {
    #[default]
    Unknown,
    JSON {
        file_path: PathBuf,
    },
    CSV {
        has_headers: bool,
        file_path: PathBuf,
    },
    TSV {
        has_headers: bool,
        file_path: PathBuf,
    },
    YahooFinanceAPI,
    JQuantsAPI,
}
