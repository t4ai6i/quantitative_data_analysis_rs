use crate::domain::models::crossover_strategy::crossover_pattern::model::CrossoverPattern;

/// A pair of `CrossoverPattern` and `rate_of_change` values.
pub(crate) struct CrossoverPatternRateOfChangePair {
    pub crossover_pattern: CrossoverPattern,
    pub rate_of_change: f64,
}

/// Analysis with combination of Crossover pattern and rate_of_change
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum AnalysisPattern {
    #[default]
    None,
    GoldenChance,
    DeadChance,
    GoldenLoss,
    DeadLoss,
}

/// Converts a `PatternRateOfChangePair` into an `AnalysisPattern`.
///
/// # Parameters
/// - `value: PatternRateOfChangePair`  
///   The input object containing a `CrossoverPattern` (either `GoldenCross` or `DeadCross`) and a `rate_of_change` value for evaluation.
///
/// # Returns
/// - `AnalysisPattern`:  
///   - Returns `AnalysisPattern::GoldenChance` if the `CrossoverPattern` is `GoldenCross` and the `rate_of_change` is greater than `0.0`.
///   - Returns `AnalysisPattern::DeadChance` if the `CrossoverPattern` is `DeadCross` and the `rate_of_change` is less than `0.0`.
///   - Returns `AnalysisPattern::GoldenLoss` if the `CrossoverPattern` is `GoldenCross` and the `rate_of_change` is less than or equal to `0.0`.
///   - Returns `AnalysisPattern::DeadLoss` if the `CrossoverPattern` is `DeadCross` and the `rate_of_change` is greater than or equal to `0.0`.
///   - Returns `AnalysisPattern::None` if none of the above conditions are met.
///
/// # Behavior
/// - The conversion logic evaluates both the `CrossoverPattern` and the `rate_of_change` value
///   based on specific conditions to determine the appropriate `AnalysisPattern` variant.
///
/// # Notes
/// - This implementation provides a structured way to classify input `PatternRateOfChangePair` values into `AnalysisPattern` variants,
///   reflecting specific conditions of the pattern and its associated rate of change.
impl From<CrossoverPatternRateOfChangePair> for AnalysisPattern {
    fn from(value: CrossoverPatternRateOfChangePair) -> Self {
        match value {
            // GoldenChance: GoldenPattern/rate_of_change > 0.0
            CrossoverPatternRateOfChangePair {
                crossover_pattern: CrossoverPattern::GoldenCross,
                rate_of_change: change,
            } if change > 0.0 => AnalysisPattern::GoldenChance,
            // DeadChance: DeadPattern/rate_of_change < 0.0
            CrossoverPatternRateOfChangePair {
                crossover_pattern: CrossoverPattern::DeadCross,
                rate_of_change: change,
            } if change < 0.0 => AnalysisPattern::DeadChance,
            // GoldenLoss: GoldenPattern/rate_of_change <= 0.0
            CrossoverPatternRateOfChangePair {
                crossover_pattern: CrossoverPattern::GoldenCross,
                rate_of_change: change,
            } if change <= 0.0 => AnalysisPattern::GoldenLoss,
            // DeadLoss: DeadPattern/rate_of_change >= 0.0
            CrossoverPatternRateOfChangePair {
                crossover_pattern: CrossoverPattern::DeadCross,
                rate_of_change: change,
            } if change >= 0.0 => AnalysisPattern::DeadLoss,
            _ => AnalysisPattern::None,
        }
    }
}
