use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Company {
    pub(crate) code: String,
    pub(crate) name: String,
    pub(crate) market: String,
}
