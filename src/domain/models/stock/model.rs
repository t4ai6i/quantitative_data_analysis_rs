use chrono::NaiveDate;
use deref_derive::{Deref, DerefMut};
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct RowStock {
    pub date: Option<NaiveDate>,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: Option<f64>,
    pub adj_close: Option<f64>,
    pub volume: Option<u64>,
}

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

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct Stocks(pub Vec<Stock>);

impl TryFrom<RowStock> for Stock {
    type Error = anyhow::Error;

    fn try_from(value: RowStock) -> Result<Self, Self::Error> {
        let RowStock {
            date,
            open,
            high,
            low,
            close,
            adj_close,
            volume,
        } = value;
        Ok(Self {
            date: date.ok_or_else(|| anyhow::anyhow!("date is None"))?,
            open: open.ok_or_else(|| anyhow::anyhow!("open is None"))?,
            high: high.ok_or_else(|| anyhow::anyhow!("high is None"))?,
            low: low.ok_or_else(|| anyhow::anyhow!("low is None"))?,
            close: close.ok_or_else(|| anyhow::anyhow!("close is None"))?,
            adj_close: adj_close.ok_or_else(|| anyhow::anyhow!("adj_close is None"))?,
            volume: volume.ok_or_else(|| anyhow::anyhow!("volume is None"))?,
        })
    }
}
