use crate::domain::models::screening::model::ScreeningResults;

pub struct Screening {
    pub screening_results: ScreeningResults,
}

impl Screening {
    pub fn new(screening_results: ScreeningResults) -> Self {
        Self { screening_results }
    }
}
