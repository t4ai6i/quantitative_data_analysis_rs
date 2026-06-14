use crate::domain::models::crossover_strategy::crossover_pattern;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CrossoverPattern(pub crossover_pattern::model::CrossoverPattern);
