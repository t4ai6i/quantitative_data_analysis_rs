use crate::presenter::trend_analysis_presenter::{DisplayCrossPattern, TrendAnalysisResponse};
use crate::presenter::trend_summary_presenter::{TrendSummaryPresenter, TrendSummaryResponse};
use crate::presenter::view_model::vec_trend_analysis_response::VecTrendAnalysisResponse;
use crate::use_case::interface::trend_summary_use_case::{TrendSummaryInput, TrendSummaryUseCase};
use anyhow::Result;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendSummaryController<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendSummaryController<'a, I, P>
where
    I: TrendSummaryUseCase,
    P: TrendSummaryPresenter,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        vec_trend_analysis_response: Vec<TrendAnalysisResponse>,
        display_cross_pattern: DisplayCrossPattern,
    ) -> Result<TrendSummaryResponse> {
        let input = TrendSummaryInput::new(VecTrendAnalysisResponse(vec_trend_analysis_response));
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output, display_cross_pattern)?;
        Ok(response)
    }
}
