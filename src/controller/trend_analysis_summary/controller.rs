use anyhow::Result;

use crate::presenter::presenters::trend_analysis::response::TrendAnalyses;
use crate::presenter::presenters::trend_analysis_summary::{presenter, response};
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::use_case::interfaces::trend_analysis_summary::{input, use_case};

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct TrendAnalysisSummary<'a, I, P> {
    interactor: &'a I,
    presenter: &'a P,
}

impl<'a, I, P> TrendAnalysisSummary<'a, I, P>
where
    I: use_case::TrendAnalysisSummary,
    P: presenter::TrendAnalysisSummary,
{
    pub fn new(interactor: &'a I, presenter: &'a P) -> Self {
        Self {
            interactor,
            presenter,
        }
    }

    pub async fn analyze(
        &self,
        trend_analyses: TrendAnalyses,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<response::TrendAnalysisSummary> {
        let input = input::TrendAnalysisSummary::new(trend_analyses);
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output, crossover_pattern_filter)?;
        Ok(response)
    }
}
