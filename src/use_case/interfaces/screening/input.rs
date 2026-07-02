use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Screening {
    pub market: String,
    pub limit: usize,
    pub preset_name: String,
    pub target_date: NaiveDate,
}

impl Screening {
    pub fn new(
        market: impl Into<String>,
        limit: usize,
        preset_name: impl Into<String>,
        target_date: NaiveDate,
    ) -> Self {
        Self {
            market: market.into(),
            limit,
            preset_name: preset_name.into(),
            target_date,
        }
    }
}
