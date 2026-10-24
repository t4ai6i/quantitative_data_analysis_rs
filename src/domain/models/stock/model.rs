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
    pub adj_open: Option<f64>,
    pub adj_high: Option<f64>,
    pub adj_low: Option<f64>,
    pub adj_close: Option<f64>,
    pub volume: Option<u64>,
    pub adj_volume: Option<f64>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Stock {
    pub date: NaiveDate,
    /// Source-provided prices; not necessarily unadjusted.
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_open: f64,
    pub adj_high: f64,
    pub adj_low: f64,
    pub adj_close: f64,
    /// Source-provided volume; not necessarily unadjusted.
    pub volume: u64,
    pub adj_volume: f64,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct Stocks(pub Vec<Stock>);

#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct BaseDatePrice {
    pub code: String,
    pub adj_close: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, Deref, DerefMut)]
pub struct BaseDatePrices(pub Vec<BaseDatePrice>);

impl TryFrom<RowStock> for Stock {
    type Error = anyhow::Error;

    fn try_from(value: RowStock) -> Result<Self, Self::Error> {
        let RowStock {
            date,
            open,
            high,
            low,
            close,
            adj_open,
            adj_high,
            adj_low,
            adj_close,
            volume,
            adj_volume,
        } = value;
        let stock = Self {
            date: date.ok_or_else(|| anyhow::anyhow!("date is None"))?,
            open: open.ok_or_else(|| anyhow::anyhow!("open is None"))?,
            high: high.ok_or_else(|| anyhow::anyhow!("high is None"))?,
            low: low.ok_or_else(|| anyhow::anyhow!("low is None"))?,
            close: close.ok_or_else(|| anyhow::anyhow!("close is None"))?,
            adj_open: adj_open.ok_or_else(|| anyhow::anyhow!("adj_open is None"))?,
            adj_high: adj_high.ok_or_else(|| anyhow::anyhow!("adj_high is None"))?,
            adj_low: adj_low.ok_or_else(|| anyhow::anyhow!("adj_low is None"))?,
            adj_close: adj_close.ok_or_else(|| anyhow::anyhow!("adj_close is None"))?,
            volume: volume.ok_or_else(|| anyhow::anyhow!("volume is None"))?,
            adj_volume: adj_volume.ok_or_else(|| anyhow::anyhow!("adj_volume is None"))?,
        };
        for (name, price) in [
            ("open", stock.open),
            ("high", stock.high),
            ("low", stock.low),
            ("close", stock.close),
            ("adj_open", stock.adj_open),
            ("adj_high", stock.adj_high),
            ("adj_low", stock.adj_low),
            ("adj_close", stock.adj_close),
        ] {
            anyhow::ensure!(price.is_finite() && price > 0.0, "invalid {name}: {price}");
        }
        anyhow::ensure!(
            stock.adj_volume.is_finite() && stock.adj_volume >= 0.0,
            "invalid adj_volume: {}",
            stock.adj_volume
        );
        Ok(stock)
    }
}
