use anyhow::Result;
use async_trait::async_trait;

use crate::presenter::presenters::trend_analysis::output;
use crate::use_case::interfaces::trend_analysis::input;

#[async_trait]
pub trait TrendAnalysis {
    async fn handle<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_BODY_MIN_RATIO: usize,
        const MARUBOZU_WICK_MAX_RATIO: usize,
        const DOJI_MAX_BODY_RATIO: usize,
    >(
        &self,
        input: input::TrendAnalysis,
    ) -> Result<
        output::TrendAnalysis<
            AFTER_DAYS,
            MARUBOZU_BODY_MIN_RATIO,
            MARUBOZU_WICK_MAX_RATIO,
            DOJI_MAX_BODY_RATIO,
        >,
    >;
}
