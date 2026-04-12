use anyhow::Result;

use crate::presenter::presenters::trend_analysis::output;
use crate::presenter::presenters::trend_analysis::presenter;
use crate::presenter::presenters::trend_analysis::response;

pub struct JSON;

impl presenter::TrendAnalysis for JSON {
    fn handle<const N: usize, const M: usize, const O: usize, const P: usize>(
        &self,
        output: output::TrendAnalysis<N, M, O, P>,
    ) -> Result<response::TrendAnalysis> {
        Ok(response::TrendAnalysis::Json {
            company: output.company,
            crossover_pattern_filter: output.crossover_pattern_filter,
            rate_of_chance: output.rate_of_chance,
            latest_chance: output.latest_chance,
            high_low_direction_signals: output.high_low_direction_signals,
            trend_reversal_analysis: output.trend_reversal_analysis,
        })
    }
}
