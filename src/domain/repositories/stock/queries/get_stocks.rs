use chrono::NaiveDate;

#[derive(Debug, Default)]
pub struct Query<'a> {
    pub code: Option<&'a str>,
    pub market: Option<&'a str>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}
