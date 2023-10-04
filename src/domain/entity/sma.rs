use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct SMA {
    pub value: f64,
    pub date: NaiveDate,
}
