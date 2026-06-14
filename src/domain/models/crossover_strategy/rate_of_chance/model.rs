#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct RateOfChance {
    pub whole: f64,
    pub golden: f64,
    pub dead: f64,
}
