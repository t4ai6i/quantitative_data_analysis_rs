use chrono::NaiveDate;

#[derive(Debug, Default)]
pub struct Query<'a> {
    pub code: Option<&'a str>,
    pub market: Option<&'a str>,
    pub target_date: NaiveDate,
}
