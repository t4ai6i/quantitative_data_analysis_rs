use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct Company {
    pub code: String,
    pub name: String,
    pub market: String,
}
