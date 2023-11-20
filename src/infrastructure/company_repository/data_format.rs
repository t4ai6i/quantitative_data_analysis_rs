pub mod csv;
pub mod json;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum DataFormat {
    #[default]
    Any,
    Json,
    CSV {
        has_headers: bool,
    },
}
