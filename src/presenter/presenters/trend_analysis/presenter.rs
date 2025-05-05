use crate::presenter::presenters::trend_analysis::output;
use crate::presenter::presenters::trend_analysis::response;
use anyhow::Result;

pub trait TrendAnalysis {
    fn handle<const N: usize, const M: usize>(
        &self,
        output: output::TrendAnalysis<N, M>,
    ) -> Result<response::TrendAnalysis>;
}
