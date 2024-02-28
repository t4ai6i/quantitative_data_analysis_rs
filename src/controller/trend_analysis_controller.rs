use crate::presenter::trend_analysis_presenter::{
    DisplayCrossPattern, TrendAnalysisPresenter, TrendAnalysisResponse,
};
use crate::use_case::interface::trend_analysis_use_case::{
    TrendAnalysisInput, TrendAnalysisUseCase,
};
use anyhow::Result;
use chrono::NaiveDate;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisController<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendAnalysisController<'a, I, P>
where
    I: TrendAnalysisUseCase,
    P: TrendAnalysisPresenter,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze<const AFTER_DAYS: usize, const FOR_DAYS: usize>(
        &self,
        code: impl Into<String>,
        market: impl Into<String>,
        start_date: NaiveDate,
        end_date: NaiveDate,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendAnalysisResponse> {
        let input =
            TrendAnalysisInput::new(code, market, start_date, end_date, display_cross_pattern);
        let output = self
            .interactor
            .handle::<AFTER_DAYS, FOR_DAYS>(input)
            .await?;
        let response = self.presenter.handle(output)?;
        Ok(response)
    }
}
