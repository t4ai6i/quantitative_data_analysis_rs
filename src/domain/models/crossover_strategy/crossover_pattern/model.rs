use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum::Display;

/// 各移動平均線が交わったときの向きのパターン
#[derive(
    Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default, Display,
)]
pub enum CrossoverPattern {
    #[default]
    Neither,
    GoldenCross,
    DeadCross,
}

/// Creates a `CrossoverPattern` from a tuple of two `Option<Ordering>` values.
///
/// # Parameters
/// - `value: (Option<Ordering>, Option<Ordering>)`
///   A tuple containing two optional `Ordering` values, which represent comparisons.
///
/// # Returns
/// - `Pattern`:
///   - Returns `CrossoverPattern::GoldenCross` if the first value is `Some(Ordering::Less)` and the second is `Some(Ordering::Greater)`.
///   - Returns `CrossoverPattern::DeadCross` if the first value is `Some(Ordering::Greater)` and the second is `Some(Ordering::Less)`.
///   - Returns `CrossoverPattern::Neither` for all other cases.
///
/// # Behavior
/// - The function pattern matches on the provided tuple of `Option<Ordering>` values to determine which `CrossoverPattern` variant to return.
///
/// # Notes
/// - Provides a convenient way to map comparison results (`Ordering`) into a `CrossoverPattern` variant.
impl From<(Option<Ordering>, Option<Ordering>)> for CrossoverPattern {
    fn from(value: (Option<Ordering>, Option<Ordering>)) -> Self {
        match value {
            (Some(Ordering::Less), Some(Ordering::Greater)) => Self::GoldenCross,
            (Some(Ordering::Greater), Some(Ordering::Less)) => Self::DeadCross,
            _ => Self::Neither,
        }
    }
}
