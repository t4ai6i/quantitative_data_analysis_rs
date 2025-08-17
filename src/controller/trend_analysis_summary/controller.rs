use anyhow::Result;

use crate::presenter::presenters::trend_analysis::response::TrendAnalyses;
use crate::presenter::presenters::trend_analysis_summary::presenter;
use crate::presenter::presenters::{trend_analysis, trend_analysis_summary};
use crate::presenter::views::shared::crossover_pattern_filter::CrossoverPatternFilter;
use crate::use_case::interfaces::trend_analysis_summary::input;
use crate::use_case::interfaces::trend_analysis_summary::use_case;

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
        vec_trend_analysis: Vec<trend_analysis::response::TrendAnalysis>,
        crossover_pattern_filter: CrossoverPatternFilter,
    ) -> Result<trend_analysis_summary::response::TrendAnalysisSummary> {
        let input = input::TrendAnalysisSummary::new(
            TrendAnalyses(vec_trend_analysis.clone()),
            vec_trend_analysis,
        );
        let output = self.interactor.handle(input).await?;
        let response = self.presenter.handle(output, crossover_pattern_filter)?;
        Ok(response)
    }
}
