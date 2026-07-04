use crate::domain::models::screening::model::ScreeningResults;
use deref_derive::{Deref, DerefMut};

pub mod json;

#[derive(Debug, Clone, PartialEq)]
pub enum Screening {
    JSON { screening_results: ScreeningResults },
}

#[derive(Debug, Clone, PartialEq, Default, Deref, DerefMut)]
pub struct Screenings(pub Vec<Screening>);
