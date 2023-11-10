pub mod csv;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum DataFormatType {
    #[default]
    Any,
    CSVFormat {
        has_headers: bool,
    },
}
