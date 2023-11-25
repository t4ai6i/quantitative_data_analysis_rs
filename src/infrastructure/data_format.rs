use std::path::PathBuf;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum DataFormat {
    #[default]
    Any,
    Json {
        file_path: PathBuf,
    },
    CSV {
        has_headers: bool,
        file_path: PathBuf,
    },
    YahooFinanceAPI,
}
