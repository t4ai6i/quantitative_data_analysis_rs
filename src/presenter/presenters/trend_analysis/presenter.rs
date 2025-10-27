use anyhow::Result;

use crate::presenter::presenters::trend_analysis::output;
use crate::presenter::presenters::trend_analysis::response;

pub trait TrendAnalysis {
    fn handle<const N: usize, const M: usize, const O: usize, const P: usize>(
        &self,
        output: output::TrendAnalysis<N, M, O, P>,
    ) -> Result<response::TrendAnalysis>;
}
