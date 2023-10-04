use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum CrossDirection {
    #[default]
    None,
    Golden,
    Dead,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Cross {
    pub date: NaiveDate,
    pub five_day: Option<f64>,
    pub twenty_five_day: Option<f64>,
    // pub seventy_five_day_sma_ave: Option<f64>,
    // pub one_hundred_day_sma_ave: Option<f64>,
    // pub two_hundred_day_sma_ave: Option<f64>,
    pub cross_direction: CrossDirection,
}
