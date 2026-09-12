use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Query {
    pub target_date: NaiveDate,
}
