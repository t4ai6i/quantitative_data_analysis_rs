use chrono::NaiveDate;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Screening {
    pub markets: Vec<String>,
    pub limit: usize,
    pub min_total_score: Option<usize>,
    pub preset_name: String,
    pub target_date: NaiveDate,
}

impl Screening {
    pub fn new(
        markets: Vec<String>,
        limit: usize,
        min_total_score: Option<usize>,
        preset_name: impl Into<String>,
        target_date: NaiveDate,
    ) -> Self {
        Self {
            markets,
            limit,
            min_total_score,
            preset_name: preset_name.into(),
            target_date,
        }
    }
}
