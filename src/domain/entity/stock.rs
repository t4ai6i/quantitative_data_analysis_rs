use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Stock {
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: u64,
}

impl Stock {
    pub fn new(
        date: NaiveDate,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        adj_close: f64,
        volume: u64,
    ) -> Self {
        Self {
            date,
            open,
            high,
            low,
            close,
            adj_close,
            volume,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct VecStock(pub Vec<Stock>);
