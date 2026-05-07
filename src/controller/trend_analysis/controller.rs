use anyhow::Result;
use chrono::NaiveDate;

use crate::presenter::presenters::trend_analysis::presenter;
use crate::presenter::presenters::trend_analysis::response;
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::use_case::interfaces::trend_analysis::input;
use crate::use_case::interfaces::trend_analysis::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysis<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendAnalysis<'a, I, P>
where
    I: use_case::TrendAnalysis,
    P: presenter::TrendAnalysis,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze<
        const AFTER_DAYS: usize,
        const FROM_END_DAYS: isize,
        const MARUBOZU_BODY_MIN_RATIO: usize,
        const MARUBOZU_WICK_MAX_RATIO: usize,
        const DOJI_MAX_BODY_RATIO: usize,
        const FAST_PERIOD: usize,
        const SLOW_PERIOD: usize,
        const SIGNAL_PERIOD: usize,
    >(
        &self,
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysis> {
        let input =
            input::TrendAnalysis::new(code, market, start_date, end_date, crossover_pattern_filter);
        let output = self
            .interactor
            .handle::<
                AFTER_DAYS,
                FROM_END_DAYS,
                MARUBOZU_BODY_MIN_RATIO,
                MARUBOZU_WICK_MAX_RATIO,
                DOJI_MAX_BODY_RATIO,
                FAST_PERIOD,
                SLOW_PERIOD,
                SIGNAL_PERIOD,
            >(
                input,
            )
            .await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
