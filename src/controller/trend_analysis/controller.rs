use anyhow::Result;
use chrono::NaiveDate;

use crate::presenter::display_macos_pattern::DisplayMACOSPattern;
use crate::presenter::trend_analysis_presenter::{TrendAnalysisPresenter, TrendAnalysisResponse};
use crate::use_case::interface::trend_analysis::input;
use crate::use_case::interface::trend_analysis::use_case;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysis<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendAnalysis<'a, I, P>
where
    I: use_case::TrendAnalysis,
    P: TrendAnalysisPresenter,
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
        const MARUBOZU_MIN_RATE: usize,
    >(
        &self,
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        display_macos_pattern: DisplayMACOSPattern,
    ) -> Result<TrendAnalysisResponse> {
        let input =
            input::TrendAnalysis::new(code, market, start_date, end_date, display_macos_pattern);
        let output = self
            .interactor
            .handle::<AFTER_DAYS, FROM_END_DAYS, MARUBOZU_MIN_RATE>(input)
            .await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
