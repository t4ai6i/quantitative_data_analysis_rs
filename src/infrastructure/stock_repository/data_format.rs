pub mod csv;
pub mod yfapi_quote;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum DataFormatType {
    #[default]
    Any,
    CSVFormat {
        has_headers: bool,
    },
    YFAPIQuote,
}
